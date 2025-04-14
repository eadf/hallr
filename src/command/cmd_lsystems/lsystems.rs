// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 lacklustr@protonmail.com https://github.com/eadf
// This file is part of the hallr crate.

use crate::HallrError;
use logos::Logos;
use std::time::{Duration, Instant};
use vector_traits::glam::{DQuat, DVec3};

pub(crate) struct Turtle {
    orientation: DQuat,
    position: DVec3,
    stack: Vec<(DQuat, DVec3)>,
    result: Vec<[DVec3; 2]>,
    /// Should the turtle draw while moving?
    pen_up: bool,
    /// should coordinates be rounded to int after each move?
    round: bool,
    sphere_radius: f64,
    //normalize_cnt:u16
}

impl Default for Turtle {
    fn default() -> Self {
        Self {
            orientation: DQuat::IDENTITY,
            position: DVec3::ZERO,
            result: Vec::new(),
            stack: Vec::new(),
            pen_up: false,
            round: false,
            sphere_radius: 1.0,
            //normalize_cnt:0,
        }
    }
}

impl Turtle {
    #[inline(always)]
    fn normalize_quaternion(&mut self) {
        //self.normalize_cnt += 1;
        //if self.normalize_cnt % 3   == 0 {
        self.orientation = self.orientation.normalize();
        //}
    }

    // Current methods reimplemented with quaternions
    fn yaw(&mut self, angle: f64) {
        let up = self.up_vector();
        let rotation = DQuat::from_axis_angle(up, angle);
        self.orientation = rotation * self.orientation;
        self.normalize_quaternion()
    }

    // Yaw: rotate heading within local tangent plane (around surface normal)
    fn geodesic_yaw(&mut self, angle: f64) {
        // self.position is normalized in each geodesic_forward()
        let normal = self.position / self.sphere_radius;
        let rotation = DQuat::from_axis_angle(normal, angle);
        self.orientation = rotation * self.orientation;
    }

    fn pitch(&mut self, angle: f64) {
        let right = self.right_vector();
        let rotation = DQuat::from_axis_angle(right, angle);
        self.orientation = rotation * self.orientation;
        self.normalize_quaternion()
    }

    fn roll(&mut self, angle: f64) {
        let forward = self.forward_vector();
        let rotation = DQuat::from_axis_angle(forward, angle);
        self.orientation = rotation * self.orientation;
        self.normalize_quaternion()
    }

    /// Rotate by yaw, pitch, roll (applied in that order) using quaternions
    fn rotate(&mut self, yaw: f64, pitch: f64, roll: f64) {
        // Create individual rotation quaternions
        let yaw_rot = DQuat::from_axis_angle(self.up_vector(), yaw);
        let pitch_rot = DQuat::from_axis_angle(self.right_vector(), pitch);
        let roll_rot = DQuat::from_axis_angle(self.forward_vector(), roll);

        // Combine rotations (note: multiplication order is right-to-left)
        self.orientation = roll_rot * pitch_rot * yaw_rot * self.orientation;
        self.normalize_quaternion()
    }

    // Euclidean forward movement
    #[inline(always)]
    fn forward(&mut self, distance: f64) {
        self.position += self.forward_vector() * distance;
    }

    // geodesic forward, hug the sphere and re-orient after move so "forward" tangents the surface
    // and "up" is the sphere radial direction.
    fn geodesic_forward(&mut self, distance: f64) {
        let angular_disp = distance / self.sphere_radius;

        let up = self.position / self.sphere_radius; // true "up"
        let forward = self.forward_vector();
        let right = up.cross(forward).normalize(); // guaranteed tangent + perpendicular
        let rotation = DQuat::from_axis_angle(right, angular_disp);

        // Rotate position around sphere center
        // self.position/sphere_radius is now normalized once per forward()
        self.position = (rotation * self.position).normalize() * self.sphere_radius;

        // Adjust orientation to maintain tangent plane
        // normalize self.orientation once per forward()
        self.orientation = (rotation * self.orientation).normalize();
    }

    #[inline(always)]
    fn forward_vector(&self) -> DVec3 {
        self.orientation * DVec3::Y
    }

    #[inline(always)]
    fn up_vector(&self) -> DVec3 {
        self.orientation * DVec3::Z
    }

    #[inline(always)]
    fn right_vector(&self) -> DVec3 {
        self.orientation * DVec3::X
    }

    /// Apply a turtle command
    fn apply(&mut self, action: &TurtleCommand) -> Result<(), HallrError> {
        match action {
            TurtleCommand::Nop => {}
            TurtleCommand::Yaw(angle) => self.yaw(*angle),
            TurtleCommand::GeodesicYaw(angle) => self.geodesic_yaw(*angle),
            TurtleCommand::Pitch(angle) => self.pitch(*angle),
            TurtleCommand::Roll(angle) => self.roll(*angle),
            TurtleCommand::Rotate(yaw, pitch, roll) => self.rotate(*yaw, *pitch, *roll),
            TurtleCommand::Forward(distance) => {
                let p0 = self.position;
                self.forward(*distance);
                if self.round {
                    self.position.x = self.position.x.round();
                    self.position.y = self.position.y.round();
                    self.position.z = self.position.z.round();
                }

                if !self.pen_up {
                    self.result.push([p0, self.position]);
                }
            }
            TurtleCommand::GeodesicForward(distance) => {
                let p0 = self.position;
                self.geodesic_forward(*distance);

                if self.round {
                    self.position.x = self.position.x.round();
                    self.position.y = self.position.y.round();
                    self.position.z = self.position.z.round();
                }

                if !self.pen_up {
                    self.result.push([p0, self.position]);
                }
            }
            TurtleCommand::PenUp => self.pen_up = true,
            TurtleCommand::PenDown => self.pen_up = false,
            TurtleCommand::Push => {
                //println!("pushing {}", self.position);
                self.stack.push((self.orientation, self.position))
            }
            TurtleCommand::Pop => {
                if let Some(pop) = self.stack.pop() {
                    // Update heading and position
                    self.orientation = pop.0;
                    self.position = pop.1;
                } else {
                    return Err(HallrError::LSystems3D("Could not pop stack".to_string()));
                }
            }
        };
        Ok(())
    }
}

pub(crate) enum TurtleCommand {
    Nop,
    Forward(f64),
    GeodesicForward(f64),
    Roll(f64),
    Pitch(f64),
    Yaw(f64),
    GeodesicYaw(f64),
    /// yaw, pitch, roll
    Rotate(f64, f64, f64),
    PenUp,
    PenDown,
    Push,
    Pop,
}

#[derive(Default)]
pub(crate) struct TurtleRules {
    rules: ahash::AHashMap<char, String>,
    axiom: String,
    tokens: ahash::AHashMap<char, TurtleCommand>,
    yaw: Option<f64>,
    pitch: Option<f64>,
    roll: Option<f64>,
    round: bool,
    iterations: u32,
    timeout: Option<Duration>,
    geodesic_radius: Option<f64>,
}

impl TurtleRules {
    pub fn add_token(&mut self, token: char, ta: TurtleCommand) -> Result<&mut Self, HallrError> {
        if self.tokens.contains_key(&token) {
            return Err(HallrError::LSystems3D(format!(
                "already contain the token {}",
                token
            )));
        }
        let _ = self.tokens.insert(token, ta);
        Ok(self)
    }

    pub fn set_timeout(&mut self, seconds: u64) -> Result<&mut Self, HallrError> {
        self.timeout = Some(Duration::from_secs(seconds));
        Ok(self)
    }

    pub fn add_axiom(&mut self, axiom: String) -> Result<&mut Self, HallrError> {
        if !self.axiom.is_empty() {
            return Err(HallrError::LSystems3D(format!(
                "already contains an axiom {}",
                axiom
            )));
        }
        // Remove spaces when adding the axiom
        self.axiom = axiom.chars().filter(|c| *c != ' ').collect();
        Ok(self)
    }

    pub fn add_rule(&mut self, rule_id: char, rule: String) -> Result<&mut Self, HallrError> {
        if rule.is_empty() {
            return Err(HallrError::LSystems3D(format!(
                "Rule too short {}",
                rule_id
            )));
        }
        // Remove spaces when adding the rule
        let cleaned_rule: String = rule.chars().filter(|c| *c != ' ').collect();

        //println!("Adding rule '{}' => '{}'", rule_id, &cleaned_rule);
        if self.rules.insert(rule_id, cleaned_rule).is_some() {
            return Err(HallrError::LSystems3D(format!(
                "Rule {} overwriting previous rule",
                rule_id
            )));
        }
        Ok(self)
    }

    /// Set the initial heading of the, not yet known, turtle
    pub fn rotate(&mut self, yaw: f64, pitch: f64, roll: f64) -> Result<&mut Self, HallrError> {
        if (yaw - 0.0).abs() > f64::EPSILON {
            self.yaw = Some(yaw);
        } else {
            self.yaw = None;
        }
        if (pitch - 0.0).abs() > f64::EPSILON {
            self.pitch = Some(pitch);
        } else {
            self.pitch = None;
        }
        if (roll - 0.0).abs() > f64::EPSILON {
            self.roll = Some(roll);
        } else {
            self.roll = None;
        }
        Ok(self)
    }
    pub fn set_geodesic_radius(&mut self, radius: f64) -> Result<&mut Self, HallrError> {
        self.geodesic_radius = Some(radius);
        Ok(self)
    }

    /// Expands the rules over the axiom 'n' times
    fn expand(&self) -> Result<Vec<char>, HallrError> {
        let start_time = Instant::now();
        let mut rv: Vec<char> = self.axiom.chars().collect();
        for i in 0..self.iterations {
            if self
                .timeout
                .is_some_and(|timeout| start_time.elapsed() > timeout)
            {
                return Err(HallrError::LSystems3D(format!(
                    "Timeout after {} seconds while processing iteration {}/{}",
                    self.timeout.unwrap().as_secs(),
                    i,
                    self.iterations
                )));
            }

            let mut tmp = Vec::<char>::with_capacity(rv.len() * 2);
            for v in rv.iter() {
                if v == &' ' {
                    continue;
                } else if let Some(rule) = self.rules.get(v) {
                    // it was a rule
                    tmp.append(&mut rule.chars().collect());
                } else {
                    // maybe a token?
                    let _ = self.tokens.get(v).ok_or_else(|| {
                        eprintln!("tokens: {:?}", self.tokens.keys());
                        eprintln!("rules: {:?}", self.rules.keys());
                        HallrError::LSystems3D(format!("Could not find rule or token:'{}'", &v))
                    })?;
                    // do not expand tokens
                    tmp.push(*v);
                }
            }
            rv = tmp;
        }
        Ok(rv)
    }

    /// sets the axioms, rules and tokens from a text string.
    pub fn parse(&mut self, cmd_custom_turtle: &str) -> Result<&mut Self, HallrError> {
        #[derive(Debug, PartialEq, Eq)]
        enum ParseTurtleAction {
            Forward,
            GeodesicForward,
            GeodesicYaw,
            Yaw,
            Pitch,
            Roll,
            // These 'states' does not carry any value, so they can be executed directly
            //Push,
            //Pop,
            //Nothing,
        }

        #[allow(clippy::upper_case_acronyms)]
        #[derive(Logos, Debug, PartialEq)]
        enum ParseToken {
            #[regex("\\.?token")]
            Token,

            #[regex("\\.?axiom")]
            Axiom,

            #[regex("\\.?rule")]
            Rule,

            #[regex("\\.?rotate")]
            Rotate,

            #[regex("\\.?yaw")]
            Yaw,

            #[regex("\\.?geodesicyaw")]
            GeodesicYaw,

            #[regex("\\.?pitch")]
            Pitch,

            #[regex("\\.?roll")]
            Roll,

            #[regex("\\.?round")]
            Round,

            #[regex("\\.?iterations")]
            Iterations,

            #[regex("\\.?timeout")]
            Timeout,

            #[regex("\\.?geodesic_radius")]
            GeodesicRadius,

            #[token("Turtle::PenUp")]
            TurtleActionPenUp,

            #[token("Turtle::PenDown")]
            TurtleActionPenDown,

            #[token("Turtle::Forward")]
            TurtleActionForward,

            #[token("Turtle::GeodesicForward")]
            TurtleActionGeodesicForward,

            #[token("Turtle::Yaw")]
            TurtleActionYaw,

            #[token("Turtle::GeodesicYaw")]
            TurtleActionGeodesicYaw,

            #[token("Turtle::Pitch")]
            TurtleActionPitch,

            #[token("Turtle::Roll")]
            TurtleActionRoll,

            #[token("Turtle::Rotate")]
            TurtleActionRotate,

            #[token("Turtle::Nop")]
            TurtleActionNop,

            #[token("Turtle::Nothing")]
            TurtleActionNothing,

            #[token("Turtle::Pop")]
            TurtleActionPop,

            #[token("Turtle::Push")]
            TurtleActionPush,

            #[token("\n")]
            EOL,

            #[regex("-?[0-9]+(.[0-9]+)?")]
            Number,

            #[regex(r#""[=<>a-zA-Z 0-9&\-+\[\]^?∧/|\\→←↑↓↻↺↕↶↷⥀⥁⇐⇒⇑⇓⇕×∘]+""#)]
            QuotedText,

            #[regex(r"[ \t\f(),;]+", logos::skip)]
            Skip,
        }

        #[derive(Debug, PartialEq)]
        enum ParseState {
            Start,
            Token(Option<char>, Option<ParseTurtleAction>),
            TokenRotate(char, Option<f64>, Option<f64>, Option<f64>),
            Axiom,
            Rule(Option<char>, Option<String>),
            Yaw,
            Rotate(Option<f64>, Option<f64>, Option<f64>),
            Iterations(Option<i32>),
            GeodesicRadius(Option<f64>),
            Timeout(Option<u64>),
        }

        println!("Will try to parse the custom 🐢: {:?}", cmd_custom_turtle);

        let mut lex = ParseToken::lexer(cmd_custom_turtle);
        let mut state = ParseState::Start;
        let mut line = 0_i32;

        while let Some(Ok(token)) = lex.next() {
            match token {
                ParseToken::Token => {
                    if state != ParseState::Start {
                        return Err(HallrError::ParseError(format!(
                            "Expected to be in Start state, was in state:{:?} when reading:{} at line {}.",
                            state,
                            lex.slice(),
                            line
                        )));
                    }
                    state = ParseState::Token(None, None);
                }
                ParseToken::Axiom => {
                    if state != ParseState::Start {
                        return Err(HallrError::ParseError(format!(
                            "Expected to be in Start state, was in state:{:?} when reading:{} at line {}.",
                            state,
                            lex.slice(),
                            line
                        )));
                    }
                    state = ParseState::Axiom;
                }
                ParseToken::Rule => {
                    if state != ParseState::Start {
                        return Err(HallrError::ParseError(format!(
                            "Expected to be in Start state, was in state:{:?} when reading:{} at line {}.",
                            state,
                            lex.slice(),
                            line
                        )));
                    }
                    state = ParseState::Rule(None, None);
                }
                ParseToken::Yaw => {
                    if state != ParseState::Start {
                        return Err(HallrError::ParseError(format!(
                            "Expected to be in Start state, was in state:{:?} when reading:{} at line {}.",
                            state,
                            lex.slice(),
                            line
                        )));
                    }
                    state = ParseState::Yaw;
                }
                ParseToken::QuotedText => {
                    let text: &str = &lex.slice()[1..lex.slice().len() - 1];
                    match state {
                        ParseState::Axiom => {
                            println!("Accepted add_axiom(\"{}\")", text);
                            let _ = self.add_axiom(text.to_string());
                            state = ParseState::Start;
                        }
                        ParseState::Token(None, None) => {
                            state = ParseState::Token(
                                Some(text.chars().next().ok_or_else(|| {
                                    HallrError::ParseError(format!(
                                        "Could not get token id as line {}",
                                        line
                                    ))
                                })?),
                                None,
                            );
                        }
                        ParseState::Rule(None, None) => {
                            if text.len() != 1 {
                                return Err(HallrError::ParseError(format!(
                                    "Rule id must be one single char, got '{}' at line {}",
                                    text, line
                                )));
                            }
                            let rule_id: char = text.chars().next().unwrap();
                            state = ParseState::Rule(Some(rule_id), None);
                        }
                        ParseState::Rule(Some(rule_id), None) => {
                            println!("Accepted add_rule('{}', \"{}\")", rule_id, text);
                            let _ = self.add_rule(rule_id, text.to_string());
                            state = ParseState::Start;
                        }
                        _ => {
                            return Err(HallrError::ParseError(format!(
                                "Bad state for QuotedText:{:?} at line {}",
                                state, line
                            )));
                        }
                    }
                }
                ParseToken::TurtleActionForward => match state {
                    ParseState::Token(Some(text), None) => {
                        state = ParseState::Token(Some(text), Some(ParseTurtleAction::Forward));
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionForward:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::TurtleActionGeodesicForward => match state {
                    ParseState::Token(Some(text), None) => {
                        state =
                            ParseState::Token(Some(text), Some(ParseTurtleAction::GeodesicForward));
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionGeodesicForward:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::TurtleActionYaw => match state {
                    ParseState::Token(Some(text), None) => {
                        state = ParseState::Token(Some(text), Some(ParseTurtleAction::Yaw));
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionYaw:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::TurtleActionGeodesicYaw => match state {
                    ParseState::Token(Some(text), None) => {
                        state = ParseState::Token(Some(text), Some(ParseTurtleAction::GeodesicYaw));
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionGeodesicYaw:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::TurtleActionPitch => match state {
                    ParseState::Token(Some(text), None) => {
                        state = ParseState::Token(Some(text), Some(ParseTurtleAction::Pitch));
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionPitch:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::TurtleActionRoll => match state {
                    ParseState::Token(Some(text), None) => {
                        state = ParseState::Token(Some(text), Some(ParseTurtleAction::Roll));
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionRoll:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::TurtleActionRotate => match state {
                    ParseState::Token(Some(text), None) => {
                        state = ParseState::TokenRotate(text, None, None, None);
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionRotate:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::TurtleActionPenDown => match state {
                    ParseState::Token(Some(text), None) => {
                        println!("Accepted add_token(\"{}\", TurtleAction::PenDown)", text);
                        let _ = self.add_token(text, TurtleCommand::PenDown);
                        state = ParseState::Start;
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionPenDown:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::TurtleActionPenUp => match state {
                    ParseState::Token(Some(text), None) => {
                        println!("Accepted add_token(\"{}\", TurtleAction::PenUp)", text);
                        let _ = self.add_token(text, TurtleCommand::PenUp);
                        state = ParseState::Start;
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionPenUp:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::TurtleActionNop | ParseToken::TurtleActionNothing => match state {
                    ParseState::Token(Some(text), None) => {
                        println!("Accepted add_token(\"{}\", TurtleAction::Nop)", text);
                        let _ = self.add_token(text, TurtleCommand::Nop);
                        state = ParseState::Start;
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionNop:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::TurtleActionPop => match state {
                    ParseState::Token(Some(text), None) => {
                        println!("Accepted add_token(\"{}\", TurtleAction::Pop)", text);
                        let _ = self.add_token(text, TurtleCommand::Pop);
                        state = ParseState::Start;
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionPop:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::TurtleActionPush => match state {
                    ParseState::Token(Some(text), None) => {
                        println!("Accepted add_token(\"{}\", TurtleAction::Push)", text);
                        let _ = self.add_token(text, TurtleCommand::Push);
                        state = ParseState::Start;
                    }
                    _ => {
                        return Err(HallrError::ParseError(format!(
                            "Bad state for TurtleActionPush:{:?} at line {}",
                            state, line
                        )));
                    }
                },
                ParseToken::Round => {
                    println!("Accepted round()");
                    self.round = true;
                    state = ParseState::Start;
                }
                ParseToken::GeodesicRadius => {
                    if state != ParseState::Start {
                        return Err(HallrError::ParseError(format!(
                            "Expected to be in Start state, was in state:{:?} when reading:{} at line {}.",
                            state,
                            lex.slice(),
                            line
                        )));
                    }
                    state = ParseState::GeodesicRadius(None);
                }
                ParseToken::EOL => {
                    line += 1;
                    state = ParseState::Start;
                }
                ParseToken::Rotate => {
                    state = ParseState::Rotate(None, None, None);
                }
                ParseToken::Iterations => {
                    if state != ParseState::Start {
                        return Err(HallrError::ParseError(format!(
                            "Expected to be in Start state, was in state:{:?} when reading:{} at line {}.",
                            state,
                            lex.slice(),
                            line
                        )));
                    }
                    state = ParseState::Iterations(None);
                }
                ParseToken::Timeout => {
                    if state != ParseState::Start {
                        return Err(HallrError::ParseError(format!(
                            "Expected to be in Start state, was in state:{:?} when reading:{} at line {}.",
                            state,
                            lex.slice(),
                            line
                        )));
                    }
                    state = ParseState::Timeout(None);
                }
                ParseToken::Number => {
                    let value = lex.slice().parse::<f64>().map_err(|e| {
                        HallrError::ParseError(format!(
                            "Could not parse number :{} at line {}. {:?}",
                            lex.slice(),
                            line,
                            e
                        ))
                    })?;

                    match state {
                        ParseState::Token(Some(text), Some(turtle)) => {
                            println!(
                                "Accepted add_token(\"{}\", TurtleAction::{:?}({}))",
                                text,
                                turtle,
                                lex.slice()
                            );

                            let _ = self.add_token(
                                text,
                                match turtle {
                                    ParseTurtleAction::GeodesicYaw => {
                                        TurtleCommand::GeodesicYaw(value.to_radians())
                                    }
                                    ParseTurtleAction::Yaw => {
                                        TurtleCommand::Yaw(value.to_radians())
                                    }
                                    ParseTurtleAction::Pitch => {
                                        TurtleCommand::Pitch(value.to_radians())
                                    }
                                    ParseTurtleAction::Roll => {
                                        TurtleCommand::Roll(value.to_radians())
                                    }
                                    ParseTurtleAction::Forward => TurtleCommand::Forward(value),
                                    ParseTurtleAction::GeodesicForward => {
                                        TurtleCommand::GeodesicForward(value)
                                    }
                                },
                            );
                            state = ParseState::Start;
                        }
                        // New cases for TokenRotate state
                        ParseState::TokenRotate(text, None, None, None) => {
                            // First parameter (yaw)
                            state = ParseState::TokenRotate(text, Some(value), None, None);
                        }
                        ParseState::TokenRotate(text, Some(yaw), None, None) => {
                            // Second parameter (pitch)
                            state = ParseState::TokenRotate(text, Some(yaw), Some(value), None);
                        }
                        ParseState::TokenRotate(text, Some(yaw), Some(pitch), None) => {
                            // Third parameter (roll) - now we have all parameters
                            let roll = value;
                            println!(
                                "Accepted add_token(\"{}\", TurtleAction::Rotate({}, {}, {}))",
                                text, yaw, pitch, roll
                            );

                            let _ = self.add_token(
                                text,
                                TurtleCommand::Rotate(
                                    yaw.to_radians(),
                                    pitch.to_radians(),
                                    roll.to_radians(),
                                ),
                            );
                            state = ParseState::Start;
                        }

                        ParseState::Rotate(None, None, None) => {
                            state = ParseState::Rotate(Some(value.to_radians()), None, None);
                        }
                        ParseState::Rotate(Some(yaw), None, None) => {
                            state = ParseState::Rotate(Some(yaw), Some(value.to_radians()), None);
                        }
                        ParseState::Rotate(Some(yaw), Some(pitch), None) => {
                            let roll = value.to_radians();
                            // present in degrees
                            println!(
                                "Accepted rotate({}, {}, {})",
                                yaw.to_degrees(),
                                pitch.to_degrees(),
                                roll.to_degrees()
                            );
                            let _ = self.rotate(yaw, pitch, roll);
                            state = ParseState::Start;
                        }
                        ParseState::Iterations(None) => {
                            let iterations = value as u32;
                            println!("Accepted iterations({})", iterations);
                            let _ = self.set_iterations(iterations);
                            state = ParseState::Start;
                        }
                        ParseState::GeodesicRadius(None) => {
                            println!("Accepted geodesic_radius({})", value);
                            let _ = self.set_geodesic_radius(value);
                            state = ParseState::Start;
                        }
                        ParseState::Timeout(None) => {
                            let seconds = value as u64;
                            println!("Accepted timeout({})", seconds);
                            let _ = self.set_timeout(seconds);
                            state = ParseState::Start;
                        }
                        _ => {
                            return Err(HallrError::ParseError(format!(
                                "Bad state for Integer:{:?} at line {}",
                                state, line
                            )));
                        }
                    }
                }
                _ => {
                    return Err(HallrError::ParseError(format!(
                        "Bad token: {:?} at line {}",
                        lex.slice(),
                        line
                    )));
                }
            }
        }
        Ok(self)
    }

    /// expands the rules and run the turtle over the result.
    pub fn exec(&self, mut turtle: Turtle) -> Result<Vec<[DVec3; 2]>, HallrError> {
        if self.round {
            turtle.round = true;
        }
        if let Some(radius) = self.geodesic_radius {
            // place turtle on the sphere
            turtle.sphere_radius = radius;
            for t in self.tokens.values() {
                if matches!(
                    t,
                    TurtleCommand::Forward(_)
                        | TurtleCommand::Pitch(_)
                        | TurtleCommand::Rotate(_, _, _)
                        | TurtleCommand::Roll(_)
                ) {
                    return Err(HallrError::ParseError(
                        "No normal forward, pitch, roll, or rotate allowed with geodesic radius"
                            .to_string(),
                    ));
                }
            }
            turtle.position = DVec3::new(0.0, 0.0, -radius);
            turtle.orientation = DQuat::IDENTITY;
        } else {
            // Apply initial rotations
            if let Some(yaw) = self.yaw {
                turtle.apply(&TurtleCommand::Yaw(yaw))?;
            }
            if let Some(roll) = self.roll {
                turtle.apply(&TurtleCommand::Roll(roll))?;
            }
            if let Some(pitch) = self.pitch {
                turtle.apply(&TurtleCommand::Pitch(pitch))?;
            }
        }

        let _start_time = Instant::now();

        let path = self.expand()?;

        #[allow(clippy::unused_enumerate_index)]
        for (_i, step) in path.iter().enumerate() {
            // Check for timeout periodically (e.g., every 1000 steps)
            // But avoid this timeout during debugging of testcases
            #[cfg(not(test))]
            if _i % 1000 == 0
                && self
                    .timeout
                    .is_some_and(|timeout| _start_time.elapsed() > timeout)
            {
                return Err(HallrError::LSystems3D(format!(
                    "Timeout after {} seconds while processing step {}/{}",
                    self.timeout.unwrap().as_secs(),
                    _i,
                    path.len()
                )));
            }
            // ’ ’ should already have been filtered out
            debug_assert_ne!(step, &' ');
            let action = self.tokens.get(step).ok_or_else(|| {
                eprintln!("tokens: {:?}", self.tokens.keys());
                eprintln!("rules: {:?}", self.rules.keys());
                HallrError::LSystems3D(format!("Could not find any rule or token:'{}'", &step))
            })?;
            turtle.apply(action)?;
        }
        Ok(turtle.result)
    }

    pub fn set_iterations(&mut self, n: u32) -> Result<&mut Self, HallrError> {
        self.iterations = n;
        Ok(self)
    }
}
