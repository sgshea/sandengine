use bevy::prelude::*;
use bevy_tnua::builtins::{TnuaBuiltinCrouch, TnuaBuiltinCrouchState, TnuaBuiltinDash};
use bevy_tnua::control_helpers::
    TnuaSimpleAirActionsCounter
;
use bevy_tnua::math::{Float, Vector3};
use bevy_tnua::prelude::*;

pub fn apply_platformer_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &CharacterMotionConfigForPlatformer,
        // This is the main component used for interacting with Tnua. It is used for both issuing
        // commands and querying the character's state.
        &mut TnuaController,
        // This is an helper for implementing one-way platforms.
        // &mut TnuaSimpleFallThroughPlatformsHelper,
        // This is an helper for implementing air actions. It counts all the air actions using a
        // single counter, so it cannot be used to implement, for example, one double jump and one
        // air dash per jump - only a single "pool" of air action "energy" shared by all air
        // actions.
        &mut TnuaSimpleAirActionsCounter,
    )>,
) {

    // Get query results
    let (
        config,
        mut controller,
        mut air_actions_counter,
    ) = query.single_mut();

    // This part is just keyboard input processing. In a real game this would probably be done
    // with a third party plugin.
    let mut direction = Vector3::ZERO;

    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction -= Vector3::X;
    }
    if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction += Vector3::X;
    }

    direction = direction.clamp_length_max(1.0);

    let jump = {
        keyboard.any_pressed([KeyCode::Space, KeyCode::ArrowUp, KeyCode::KeyW])
    };
    let dash = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    // This needs to be called once per frame. It lets the air actions counter know about the
    // air status of the character. Specifically:
    // * Is it grounded or is it midair?
    // * Did any air action just start?
    // * Did any air action just finished?
    // * Is any air action currently ongoing?
    air_actions_counter.update(controller.as_mut());

    let speed_factor =
        // `TnuaController::concrete_action` can be used to determine if an action is currently
        // running, and query its status. Here, we use it to check if the character is
        // currently crouching, so that we can limit its speed.
        if let Some((_, state)) = controller.concrete_action::<TnuaBuiltinCrouch>() {
            // If the crouch is finished (last stages of standing up) we don't need to slow the
            // character down.
            if matches!(state, TnuaBuiltinCrouchState::Rising) {
                1.0
            } else {
                0.2
            }
        } else {
            1.0
        };

    // The basis is Tnua's most fundamental control command, governing over the character's
    // regular movement. The basis (and, to some extent, the actions as well) contains both
    // configuration - which in this case we copy over from `config.walk` - and controls like
    // `desired_velocity` or `desired_forward` which we compute here based on the current
    // frame's input.
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: {
            direction * speed_factor * config.speed
        },
        desired_forward: {
            // For platformers, we only want ot change direction when the character tries to
            // moves (or when the player explicitly wants to set the direction)
            direction.normalize_or_zero()
        },
        ..config.walk.clone()
    });

    // if crouch {
    //     // Crouching is an action. We either feed it or we don't - other than that there is
    //     // nothing to set from the current frame's input. We do pass it through the crouch
    //     // enforcer though, which makes sure the character does not stand up if below an
    //     // obstacle.
    //     controller.action(crouch_enforcer.enforcing(config.crouch.clone()));
    // }

    if jump {
        controller.action(TnuaBuiltinJump {
            // Jumping, like crouching, is an action that we either feed or don't. However,
            // because it can be used in midair, we want to set its `allow_in_air`. The air
            // counter helps us with that.
            //
            // The air actions counter is used to decide if the action is allowed midair by
            // determining how many actions were performed since the last time the character
            // was considered "grounded" - including the first jump (if it was done from the
            // ground) or the initiation of a free fall.
            //
            // `air_count_for` needs the name of the action to be performed (in this case
            // `TnuaBuiltinJump::NAME`) because if the player is still holding the jump button,
            // we want it to be considered as the same air action number. So, if the player
            // performs an air jump, before the air jump `air_count_for` will return 1 for any
            // action, but after it it'll return 1 only for `TnuaBuiltinJump::NAME`
            // (maintaining the jump) and 2 for any other action. Of course, if the player
            // releases the button and presses it again it'll return 2.
            allow_in_air: air_actions_counter.air_count_for(TnuaBuiltinJump::NAME)
                <= config.actions_in_air,
            ..config.jump.clone()
        });
    }

    if dash {
        controller.action(TnuaBuiltinDash {
            // Dashing is also an action, but because it has directions we need to provide said
            // directions. `displacement` is a vector that determines where the jump will bring
            // us. Note that even after reaching the displacement, the character may still have
            // some leftover velocity (configurable with the other parameters of the action)
            //
            // The displacement is "frozen" when the action starts - user code does not have to
            // worry about storing the original direction.
            displacement: direction.normalize() * config.dash_distance,
            // When set, the `desired_forward` of the dash action "overrides" the
            // `desired_forward` of the walk basis. Like the displacement, it gets "frozen" -
            // allowing to easily maintain a forward direction during the dash.
            desired_forward: {
                direction.normalize()
            },
            allow_in_air: air_actions_counter.air_count_for(TnuaBuiltinDash::NAME)
                <= config.actions_in_air,
            ..config.dash.clone()
        });
    }
}

#[derive(Component)]
pub struct CharacterMotionConfigForPlatformer {
    pub speed: Float,
    pub walk: TnuaBuiltinWalk,
    pub actions_in_air: usize,
    pub jump: TnuaBuiltinJump,
    pub crouch: TnuaBuiltinCrouch,
    pub dash_distance: Float,
    pub dash: TnuaBuiltinDash,
    pub one_way_platforms_min_proximity: Float,
}