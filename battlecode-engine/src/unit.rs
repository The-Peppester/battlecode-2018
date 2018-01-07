//! Units are player-controlled entities that can perform certain
//! game actions, depending on their type.

use failure::Error;
use std::cmp;

use super::constants::*;
use super::error::GameError;
use super::location::*;
use super::research::Level;
use super::world::*;
use unit::UnitType::*;
use unit::Location::*;

/// Percentage.
pub type Percent = u32;

/// The ID of an unit is assigned when the unit is spawned.
pub type UnitID = u16;

/// The public version of the unit. Contains all the unit's stats but none of
/// the action. The other team can see everything in the unit info.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UnitInfo {
    /// The unique ID of the unit.
    pub id: UnitID,
    /// The team the unit is on.
    pub team: Team,
    /// The type of the unit.
    pub unit_type: UnitType,
    /// The current location of the unit.
    pub location: Location,
    /// The current health of the unit.
    pub health: u32,
}

/// The different unit types, which include factories, rockets, and the robots.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UnitType {
    /// Workers are the foundation of the civilization.
    Worker,
    /// Knights are a melee unit that is strong in numbers.
    Knight,
    /// Rangers are a ranged unit with good all-around combat.
    Ranger,
    /// Mages are a fragile but specialized ranged unit for large areas.
    Mage,
    /// Healers are a suport unit that can heal other units.
    Healer,
    /// Factories are the hub for producing combative robots.
    Factory,
    /// Rockets are the only unit that can move between planets.
    Rocket,
}

impl UnitType {
    /// List all the unit types.
    pub fn all() -> Vec<UnitType> {
        vec![
            Worker,
            Knight,
            Ranger,
            Mage,
            Healer,
            Factory,
            Rocket,
        ]
    }

    /// Return the default stats of the given unit type.
    fn default(&self) -> Unit {
        match *self {
            Worker => Unit {
                unit_type: Worker,
                health: 100,
                max_health: 100,
                damage: 0,
                attack_range: 0,
                vision_range: 50,
                movement_cooldown: 20,
                attack_cooldown: 0,
                ability_cooldown: 50,
                ..Default::default()
            },
            Knight => Unit {
                unit_type: Knight,
                health: 250,
                max_health: 250,
                damage: 100,
                attack_range: 1,
                vision_range: 50,
                movement_cooldown: 15,
                attack_cooldown: 20,
                ability_cooldown: 75,
                ability_range: 10,
                ..Default::default()
            },
            Ranger => Unit {
                unit_type: Ranger,
                health: 200,
                max_health: 200,
                damage: 70,
                attack_range: 50,
                vision_range: 70,
                movement_cooldown: 20,
                attack_cooldown: 20,
                ability_cooldown: 150,
                ability_range: 10,
                ..Default::default()
            },
            Mage => Unit {
                unit_type: Mage,
                health: 100,
                max_health: 100,
                damage: 150,
                attack_range: 30,
                vision_range: 30,
                movement_cooldown: 20,
                attack_cooldown: 20,
                ability_cooldown: 100,
                ability_range: 5,
                ..Default::default()
            },
            Healer => Unit {
                unit_type: Healer,
                health: 100,
                max_health: 100,
                damage: -10,
                attack_range: 30,
                vision_range: 50,
                movement_cooldown: 25,
                attack_cooldown: 10,
                ability_cooldown: 50,
                ability_range: 30,
                ..Default::default()
            },
            Factory => Unit {
                unit_type: Factory,
                health: 1000 / 4,
                max_health: 1000,
                ..Default::default()
            },
            Rocket => Unit {
                unit_type: Rocket,
                health: 200 / 4,
                max_health: 200,
                ..Default::default()
            },
        }
    }

    /// Whether the unit type is a robot.
    pub fn is_robot(&self) -> bool {
        match *self {
            Worker => true,
            Knight => true,
            Ranger => true,
            Mage => true,
            Healer => true,
            Factory => false,
            Rocket => false,
        }
    }

    /// Whether the unit type is a structure.
    pub fn is_structure(&self) -> bool {
        match *self {
            Worker => false,
            Knight => false,
            Ranger => false,
            Mage => false,
            Healer => false,
            Factory => true,
            Rocket => true,
        }
    }

    /// The cost of the unit in a factory.
    ///
    /// Errors if the unit cannot be produced in a factory.
    pub fn factory_cost(self) -> Result<u32, Error> {
        match self {
            UnitType::Worker => Ok(FACTORY_WORKER_COST),
            UnitType::Knight => Ok(FACTORY_KNIGHT_COST),
            UnitType::Ranger => Ok(FACTORY_RANGER_COST),
            UnitType::Mage => Ok(FACTORY_MAGE_COST),
            UnitType::Healer => Ok(FACTORY_HEALER_COST),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// The cost to blueprint the unit.
    ///
    /// Errors if the unit cannot be blueprinted.
    pub fn blueprint_cost(self) -> Result<u32, Error> {
        match self {
            UnitType::Factory => Ok(BLUEPRINT_FACTORY_COST),
            UnitType::Rocket => Ok(BLUEPRINT_ROCKET_COST),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// The cost to replicate the unit.
    ///
    /// Errors if the unit cannot be replicated.
    pub fn replicate_cost(self) -> Result<u32, Error> {
        match self {
            UnitType::Worker => Ok(FACTORY_WORKER_COST),
            _ => Err(GameError::InappropriateUnitType)?,
        }
    }

    /// The value of a unit, as relevant to tiebreakers.
    pub fn value(self) -> u32 {
        match self {
            UnitType::Worker => FACTORY_WORKER_COST,
            UnitType::Knight => FACTORY_KNIGHT_COST,
            UnitType::Ranger => FACTORY_RANGER_COST,
            UnitType::Mage => FACTORY_MAGE_COST,
            UnitType::Healer => FACTORY_HEALER_COST,
            UnitType::Factory => BLUEPRINT_FACTORY_COST,
            UnitType::Rocket => BLUEPRINT_ROCKET_COST,
        }
    }
}

/// A single unit in the game and all its associated properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Unit {
    // All units.
    id: UnitID,
    team: Team,
    level: Level,
    unit_type: UnitType,
    location: Location,
    health: u32,
    max_health: u32,
    vision_range: u32,

    // All robots.
    damage: i32,
    attack_range: u32,
    movement_heat: u32,
    attack_heat: u32,
    movement_cooldown: u32,
    attack_cooldown: u32,

    /// The special ability for each non-worker robot:
    /// Javelin, Snipe, Blink, Overcharge.
    is_ability_unlocked: bool,
    ability_heat: u32,
    ability_cooldown: u32,
    ability_range: u32,

    // Worker special ability.
    has_worker_acted: bool,
    build_health: u32,
    harvest_amount: u32,

    // Knight special ability.
    defense: u32,

    // Ranger special ability.
    cannot_attack_range: u32,
    countdown: u32,
    target_location: Option<MapLocation>,
    is_sniping: bool,

    // Healer special ability.
    self_heal_amount: u32,

    // Structures.
    is_built: bool,
    max_capacity: usize,
    garrison: Vec<UnitID>,

    // Factory special ability.
    factory_unit_type: Option<UnitType>,
    factory_rounds_left: Option<Rounds>,

    // Rocket special ability.
    is_used: bool,
    travel_time_decrease: Percent,
}

impl Default for Unit {
    fn default() -> Unit {
        Unit {
            id: 0,
            health: 0,
            location: Unknown,
            max_health: 0,
            team: Team::Red,
            unit_type: Worker,

            level: 0,
            movement_heat: 0,
            attack_heat: 0,
            damage: 0,
            attack_range: 0,
            vision_range: 0,
            movement_cooldown: 0,
            attack_cooldown: 0,

            is_ability_unlocked: false,
            ability_heat: 0,
            ability_cooldown: 0,
            ability_range: 0,

            is_built: false,
            max_capacity: 8,
            garrison: vec![],
            has_worker_acted: false,
            build_health: 5,
            harvest_amount: 3,
            defense: 5,
            cannot_attack_range: 10,
            countdown: 0,
            target_location: None,
            is_sniping: false,
            self_heal_amount: 1,
            factory_unit_type: None,
            factory_rounds_left: None,
            is_used: false,
            travel_time_decrease: 100,
        }
    }
}

impl Unit {
    /// Create a new unit of the given type.
    pub(crate) fn new(id: UnitID,
               team: Team,
               unit_type: UnitType,
               level: Level,
               location: Location) -> Result<Unit, Error> {
        let mut unit = unit_type.default();
        unit.id = id;
        unit.team = team;
        unit.location = location;

        for _ in 0..level {
            unit.research()?;
        }
        Ok(unit)
    }

    /// The public version of the unit.
    pub(crate) fn info(&self) -> UnitInfo {
        UnitInfo {
            id: self.id,
            team: self.team,
            unit_type: self.unit_type,
            location: self.location,
            health: self.health,
        }
    }

    // ************************************************************************
    // ***************************** GENERAL METHODS **************************
    // ************************************************************************

    /// The unique ID of a unit.
    pub fn id(&self) -> UnitID {
        self.id
    }

    /// The team the unit belongs to.
    pub fn team(&self) -> Team {
        self.team
    }

    /// The current research level.
    pub fn research_level(&self) -> Level {
        self.level
    }

    /// The unit type.
    pub fn unit_type(&self) -> UnitType {
        self.unit_type
    }

    /// The location of the unit.
    pub fn location(&self) -> Location {
        self.location
    }

    /// The current health.
    pub fn health(&self) -> u32 {
        self.health
    }

    /// The maximum health.
    pub fn max_health(&self) -> u32 {
        self.max_health
    }

    /// The unit vision range.
    pub fn vision_range(&self) -> u32 {
        self.vision_range
    }

    // ************************************************************************
    // *************************** HELPER METHODS *****************************
    // ************************************************************************

    /// Ok if the unit is a robot. Errors otherwise.
    fn ok_if_robot(&self) -> Result<(), Error> {
        if self.unit_type.is_robot() {
            Ok(())
        } else {
            Err(GameError::InappropriateUnitType)?
        }
    }

    /// Ok if the unit is a structure. Errors otherwise.
    fn ok_if_structure(&self) -> Result<(), Error> {
        if self.unit_type.is_structure() {
            Ok(())
        } else {
            Err(GameError::InappropriateUnitType)?
        }
    }

    /// Ok if the unit is the given type. Errors otherwise.
    fn ok_if_unit_type(&self, unit_type: UnitType) -> Result<(), Error> {
        if self.unit_type == unit_type {
            Ok(())
        } else {
            Err(GameError::InappropriateUnitType)?
        }
    }

    // ************************************************************************
    // ****************** ROBOT MOVEMENT / COMBAT METHODS *********************
    // ************************************************************************

    /// The damage inflicted by the robot during a normal attack.
    ///
    /// Errors if the unit is not a robot.
    pub fn damage(&self) -> Result<i32, Error> {
        self.ok_if_robot()?;
        Ok(self.damage)
    }

    /// The attack range.
    ///
    /// Errors if the unit is not a robot.
    pub fn attack_range(&self) -> Result<u32, Error> {
        self.ok_if_robot()?;
        Ok(self.attack_range)
    }

    /// The movement heat.
    ///
    /// Errors if the unit is not a robot.
    pub fn movement_heat(&self) -> Result<u32, Error> {
        self.ok_if_robot()?;
        Ok(self.movement_heat)
    }

    /// The attack heat.
    ///
    /// Errors if the unit is not a robot.
    pub fn attack_heat(&self) -> Result<u32, Error> {
        self.ok_if_robot()?;
        Ok(self.attack_heat)
    }

    /// The movement cooldown.
    ///
    /// Errors if the unit is not a robot.
    pub fn movement_cooldown(&self) -> Result<u32, Error> {
        self.ok_if_robot()?;
        Ok(self.movement_cooldown)
    }

    /// The attack cooldown.
    ///
    /// Errors if the unit is not a robot.
    pub fn attack_cooldown(&self) -> Result<u32, Error> {
        self.ok_if_robot()?;
        Ok(self.attack_cooldown)
    }

    /// Whether the unit is ready to move. The movement heat must be lower than
    /// the maximum heat to attack.
    ///
    /// Errors if the unit is not a robot.
    pub(crate) fn is_move_ready(&self) -> Result<bool, Error> {
        Ok(self.movement_heat()? < MAX_HEAT_TO_ACT)
    }

    /// Updates the unit's location as it if has moved, and increases the
    /// movement heat.
    /// 
    /// Errors if the unit is not a robot, or not ready to move.
    pub(crate) fn move_to(&mut self, location: MapLocation)
                   -> Result<(), Error> {
        if self.is_move_ready()? {
            self.movement_heat += self.movement_cooldown;
            self.location = OnMap(location);
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Tests whether the robot can attack the target location.
    ///
    /// Errors if the unit is not a robot.
    pub(crate) fn is_within_attack_range(&self, target_loc: Location) -> Result<bool, Error> {
        self.ok_if_robot()?;
        if self.unit_type() == UnitType::Ranger {
            if self.location().is_within_range(self.cannot_attack_range, target_loc) {
                return Ok(false);
            }
        }
        Ok(self.location().is_within_range(self.attack_range()?, target_loc))
    }

    /// Whether the unit is ready to attack. The attack heat must be lower than
    /// the maximum heat to act.
    ///
    /// Errors if the unit is not a robot.
    pub(crate) fn is_attack_ready(&self) -> Result<bool, Error> {
        Ok(self.attack_heat()? < MAX_HEAT_TO_ACT)
    }

    /// Updates the unit as if it has attacked, and increases the attack heat.
    /// Returns the damage done.
    ///
    /// Errors if the unit is not a robot, or not ready to attack.
    pub(crate) fn use_attack(&mut self) -> Result<i32, Error> {
        if self.is_attack_ready()? {
            self.attack_heat += self.attack_cooldown;
            Ok(self.damage)
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Take the amount of damage given, returning true if the unit has died.
    /// Returns false if the unit is still alive.
    pub(crate) fn take_damage(&mut self, mut damage: i32) -> bool {
        if damage < 0 {
            self.be_healed((-damage) as u32);
            return false;
        }
        if self.unit_type() == UnitType::Knight {
            damage = cmp::max(damage - self.defense as i32, 0);
        }
        self.health -= cmp::min(damage, self.health as i32) as u32;
        self.health == 0
    }

    /// Increases the unit's current health by the given amount, without healing
    /// beyond the unit's maximum health. Returns true if unit is healed to max.
    pub(crate) fn be_healed(&mut self, heal_amount: u32) -> bool {
        self.health = cmp::min(self.health + heal_amount, self.max_health);
        self.health == self.max_health
    }

    // ************************************************************************
    // *************************** ABILITY METHODS *****************************
    // ************************************************************************
    
    /// Whether the active ability is unlocked.
    ///
    /// Errors if the unit is not a robot.
    pub fn is_ability_unlocked(&self) -> Result<bool, Error> {
        self.ok_if_robot()?;
        Ok(self.is_ability_unlocked)
    }

    /// The active ability heat.
    ///
    /// Errors if the unit is not a robot.
    pub fn ability_heat(&self) -> Result<u32, Error>{
        self.ok_if_robot()?;
        Ok(self.ability_heat)
    }

    /// The active ability cooldown.
    ///
    /// Errors if the unit is not a robot.
    pub fn ability_cooldown(&self) -> Result<u32, Error>{
        self.ok_if_robot()?;
        Ok(self.ability_cooldown)
    }

    /// The active ability range.
    /// 
    /// Errors if the unit is not a robot.
    pub fn ability_range(&self) -> Result<u32, Error> {
        self.ok_if_robot()?;
        Ok(self.ability_range)
    }

    /// Ok if unit can use ability.
    ///
    /// Errors if the unit is not a robot, has insufficient research level
    /// or if unit is a worker.
    pub(crate) fn ok_if_ability(&self) -> Result<(), Error> {
        self.ok_if_robot()?;
        if !self.is_ability_unlocked()? {
            Err(GameError::InvalidResearchLevel)?
        }

        if self.unit_type == Worker {
            Err(GameError::InappropriateUnitType)?
        }

        Ok(())
    }

    /// Whether the unit can use its ability. The unit's ability heat must 
    /// be lower than the maximum heat to act. 
    ///
    /// Errors if the unit is not a robot
    pub(crate) fn is_ability_ready(&self) -> Result<bool, Error> {
        Ok(self.ability_heat()? < MAX_HEAT_TO_ACT)
    }

    /// Resets a unit's ability cooldown.
    /// 
    /// Errors if the unit is not a robot. 
    pub(crate) fn reset_ability_cooldown(&mut self) -> Result<(), Error> {
        self.ok_if_robot()?; 
        self.ability_heat = MIN_HEAT;
        Ok(())
    }

    // ************************************************************************
    // *************************** WORKER METHODS *****************************
    // ************************************************************************

    /// Whether the worker has already acted (harveted, blueprinted, built, or
    /// repaired) this round.
    ///
    /// Errors if the unit is not a worker.
    pub fn worker_has_acted(&self) -> Result<bool, Error> {
        self.ok_if_unit_type(Worker)?;
        Ok(self.has_worker_acted)
    }

    /// The health restored when building or repairing a structure.
    ///
    /// Errors if the unit is not a worker.
    pub fn worker_build_health(&self) -> Result<u32, Error> {
        self.ok_if_unit_type(Worker)?;
        Ok(self.build_health)
    }

    /// The maximum amount of karbonite harvested from a deposit in one turn.
    ///
    /// Errors if the unit is not a worker.
    pub fn worker_harvest_amount(&self) -> Result<u32, Error> {
        self.ok_if_unit_type(Worker)?;
        Ok(self.harvest_amount)
    }

    /// Whether the unit can perform a worker action (building, blueprinting, 
    /// harvesting, or replicating).
    ///
    /// Errors if the unit is not a worker.
    pub(crate) fn can_worker_act(&self) -> Result<bool, Error> {
        self.ok_if_unit_type(Worker)?;
        Ok(!self.has_worker_acted)
    }

    /// Updates the unit as if it has performed a worker action.
    ///
    /// Errors if the unit is not a worker, or has already acted.
    pub(crate) fn worker_act(&mut self) -> Result<(), Error> {
        if self.can_worker_act()? {
            self.has_worker_acted = true;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Updates the worker as though it has replicated. In reality,
    /// just updates the worker's ability heat.
    pub(crate) fn replicate(&mut self) {
        self.ability_heat += self.ability_cooldown;
    }

    // ************************************************************************
    // *************************** KNIGHT METHODS *****************************
    // ************************************************************************

    /// The amount of damage resisted by a knight when attacked.
    ///
    /// Errors if the unit is not a knight.
    pub fn knight_defense(&self) -> Result<u32, Error> {
        self.ok_if_unit_type(Knight)?;
        Ok(self.defense)
    }

    /// Ok if the unit can javelin. 
    /// 
    /// Errors if the unit is not a knight, or has insufficient research level
    pub(crate) fn ok_if_javelin(&self) -> Result<(), Error> {
        self.ok_if_unit_type(Knight)?;
        Ok(self.ok_if_ability()?)
    }

    /// Updates the unit as if it has javelined.
    /// 
    /// Errors if the unit is not a knight, or not ready to javelin.
    pub(crate) fn javelin(&mut self) -> Result<(i32), Error> {
        self.ok_if_javelin()?;
        if self.is_ability_ready()? {
            self.ability_heat += self.ability_cooldown;
            Ok(self.damage)
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    // ************************************************************************
    // *************************** RANGER METHODS *****************************
    // ************************************************************************

    /// The range within a ranger cannot attack.
    ///
    /// Errors if the unit is not a ranger.
    pub fn ranger_cannot_attack_range(&self) -> Result<u32, Error> {
        self.ok_if_unit_type(Ranger)?;
        Ok(self.cannot_attack_range)
    }

    /// The countdown for ranger's snipe.
    ///
    /// Errors if the unit is not a ranger.
    pub fn ranger_countdown(&self) -> Result<u32, Error> {
        self.ok_if_unit_type(Ranger)?;
        Ok(self.countdown)
    }

    /// The target location for ranger's snipe.
    ///
    /// Errors if the unit is not a ranger.
    pub fn ranger_target_location(&self) -> Result<Option<MapLocation>, Error> {
        self.ok_if_unit_type(Ranger)?;
        Ok(self.target_location)
    }

    /// Whether the ranger is sniping.
    ///
    /// Errors if the unit is not a ranger.
    pub fn ranger_is_sniping(&self) -> Result<bool, Error> {
        self.ok_if_unit_type(Ranger)?;
        Ok(self.is_sniping)
    }

    /// Ok if the unit can snipe
    /// 
    /// Errors if the unit is not a ranger, or has insufficient research level
    pub(crate) fn ok_if_snipe(&self) -> Result<(), Error> {
        self.ok_if_unit_type(Ranger)?;
        Ok(self.ok_if_ability()?)
    }

    /// Whether the unit is ready to process sniping.
    ///
    /// Errors if the unit is not ready to snipe.
    fn is_process_snipe_ready(&self) -> bool {
        self.is_sniping
        && self.countdown == 0
        && self.target_location.is_some()
    }

    /// Updates the unit as if it has begun sniping. The unit's ability heat 
    /// does not increase until it has sniped.
    ///
    /// Errors if the unit is not a ranger, or not ready to begin sniping. 
    pub(crate) fn begin_snipe(&mut self, location: MapLocation) -> Result<(), Error> {
        self.ok_if_snipe()?;
        if self.is_ability_ready()? {
            self.movement_heat = u32::max_value();
            self.attack_heat = u32::max_value();
            self.target_location = Some(location);
            self.countdown = MAX_RANGER_COUNTDOWN;
            self.is_sniping = true;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Updates the unit as if it has sniped.
    pub(crate) fn process_snipe(&mut self) -> Option<MapLocation> {
        if self.is_process_snipe_ready() {
            self.attack_heat = MIN_HEAT;
            self.movement_heat = MIN_HEAT;
            self.ability_heat += self.ability_cooldown;
            self.is_sniping = false;
            self.target_location
        } else {
            None
        }
    }

    // ************************************************************************
    // **************************** MAGE METHODS ******************************
    // ************************************************************************

    /// Ok if the unit can blink.
    /// 
    /// Errors if the unit is not a mage, or insufficient research level. 
    pub(crate) fn ok_if_blink(&self) -> Result<(), Error> {
        self.ok_if_unit_type(Mage)?;
        Ok(self.ok_if_ability()?)
    }

    /// Updates the unit as if it has blinked.
    pub(crate) fn blink(&mut self, location: MapLocation) {
        self.ability_heat += self.ability_cooldown;
        self.location = OnMap(location);
    }

    // ************************************************************************
    // *************************** HEALER METHODS *****************************
    // ************************************************************************

    /// The amount of health passively restored to itself each round.
    ///
    /// Errors if the unit is not a healer.
    pub fn healer_self_heal_amount(&self) -> Result<u32, Error> {
        self.ok_if_unit_type(Healer)?;
        Ok(self.self_heal_amount)
    }

    /// Ok if the unit can overcharge
    /// 
    /// Errors if the unit is not a healer.
    pub(crate) fn ok_if_overcharge(&self) -> Result<(), Error> {
        self.ok_if_unit_type(Healer)?;
        Ok(self.ok_if_ability()?)
    }

    /// Updates the unit as if it has overcharged.
    /// 
    /// Errors if the unit is not a healer, or not ready to overcharge.
    pub(crate) fn overcharge(&mut self) -> Result<(), Error> {
        self.ok_if_overcharge()?;
        if  self.is_ability_ready()? {
            self.ability_heat += self.ability_cooldown;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    // ************************************************************************
    // ************************* STRUCTURE METHODS ****************************
    // ************************************************************************

    /// Whether this structure has been built.
    ///
    /// Errors if the unit is not a structure.
    pub fn structure_is_built(&self) -> Result<bool, Error> {
        self.ok_if_structure()?;
        Ok(self.is_built)
    }

    /// The max capacity of a structure.
    ///
    /// Errors if the unit is not a structure.
    pub fn structure_max_capacity(&self) -> Result<usize, Error> {
        self.ok_if_structure()?;
        Ok(self.max_capacity)
    }

    /// Returns the units in the structure's garrison.
    ///
    /// Errors if the unit is not a structure.
    pub fn structure_garrison(&self) -> Result<Vec<UnitID>, Error> {
        self.ok_if_structure()?;
        Ok(self.garrison.clone())
    }

    /// Updates this structure as though a worker has just built it. Only errors
    /// if the unit is not a structure (i.e. does not check that structure is
    /// incomplete).
    pub(crate) fn be_built(&mut self, build_health: u32) -> Result<(), Error> {
        self.ok_if_structure()?;
        if self.be_healed(build_health) {
            self.is_built = true;
        }
        Ok(())
    }

    /// Returns OK if the structure can load a unit. The structure 
    /// must have enough space.
    ///
    /// Errors if the action is not possible.
    pub(crate) fn ok_if_can_load(&self) -> Result<(), Error> {
        self.ok_if_structure()?;
        if self.structure_garrison()?.len() == self.structure_max_capacity()? {
            Err(GameError::NotEnoughSpace)?;
        }
        Ok(())
    }

    /// Updates the structure as if it has loaded a unit inside its garrison.
    /// Adds the unit ID to the garrison.
    ///
    /// Errors if this unit is not a structure, or it cannot load.
    pub(crate) fn load(&mut self, id: UnitID) -> Result<(), Error> {
        self.ok_if_can_load()?;
        self.ok_if_structure()?;
        self.garrison.push(id);
        Ok(())
    }

    /// Returns OK if the structure can unload a unit. The structure must be on
    /// a planet and it must have at least one unit to unload. Does not check
    /// whether the unit is ready to move.
    ///
    /// Returns if the action is not possible.
    pub(crate) fn ok_if_can_unload_unit(&self) -> Result<(), Error> {
        match self.location() {
            OnMap(_) => {
                if self.structure_garrison()?.len() == 0 {
                    Err(GameError::GarrisonEmpty)?;
                }
                Ok(())
            },
            _ => Err(GameError::UnitNotOnMap)?,
        }
    }

    /// Updates the structure as if it has unloaded a single unit from the
    /// structure, returning the unit ID.
    ///
    /// Errors if the unit is not a structure, or it cannot unload.
    pub(crate) fn unload_unit(&mut self) -> Result<UnitID, Error> {
        self.ok_if_can_unload_unit()?;
        Ok(self.garrison.remove(0))
    }

    // ************************************************************************
    // ************************** FACTORY METHODS *****************************
    // ************************************************************************

    /// The unit type currently being produced by the factory, or None if the
    /// factory is not producing a unit.
    ///
    /// Errors if the unit is not a factory
    pub fn factory_unit_type(&self) -> Result<Option<UnitType>, Error> {
        self.ok_if_unit_type(Factory)?;
        Ok(self.factory_unit_type)
    }

    /// The number of rounds left to produce a robot in this factory. Returns
    /// None if no unit is currently being produced.
    ///
    /// Errors if the unit is not a factory.
    pub fn factory_rounds_left(&self) -> Result<Option<Rounds>, Error> {
        self.ok_if_unit_type(Factory)?;
        Ok(self.factory_rounds_left)
    }

    /// Returns OK if the factory can produce a robot of this type.
    ///
    /// Errors if the action is not possible.
    pub(crate) fn ok_if_can_produce_robot(&self, unit_type: UnitType) -> Result<(), Error> {
        self.ok_if_unit_type(Factory)?;
        if !unit_type.is_robot() {
            Err(GameError::InappropriateUnitType)?;
        }
        if !self.factory_unit_type.is_none() {
            Err(GameError::FactoryBusy)?;
        }
        Ok(())
    }

    /// Starts producing a robot of this type.
    /// Assumes the unit can produce a robot.
    pub(crate) fn produce_robot(&mut self, unit_type: UnitType) {
        self.factory_unit_type = Some(unit_type);
        self.factory_rounds_left = Some(FACTORY_NUM_ROUNDS);
    }

    /// Ends a round for this factory. If the factory is currently producing a
    /// robot, decreases the number of rounds left. If the number of rounds is
    /// 0 and the factory can load a unit into the garrison, loads the unit and
    /// returns the unit type and resets the factory. If the factory cannot
    /// load a unit, does nothing.
    ///
    /// Assumes the unit is a factory.
    pub(crate) fn process_factory_round(&mut self) -> Option<UnitType> {
        if self.factory_rounds_left.is_none() {
            return None;
        }

        let rounds_left = self.factory_rounds_left.unwrap() - 1;
        if rounds_left != 0 {
            self.factory_rounds_left = Some(rounds_left);
            return None;
        }

        if self.ok_if_can_load().is_err() {
            return None;
        }

        let unit_type = self.factory_unit_type.unwrap();
        self.factory_rounds_left = None;
        self.factory_unit_type = None;
        Some(unit_type)
    }

    // ************************************************************************
    // *************************** ROCKET METHODS *****************************
    // ************************************************************************

    /// Whether the rocket has already been used.
    ///
    /// Errors if the unit is not a rocket.
    pub fn rocket_is_used(&self) -> Result<bool, Error> {
        self.ok_if_unit_type(Rocket)?;
        Ok(self.is_used)
    }

    /// The number of rounds the rocket travel time is reduced by compared
    /// to the travel time determined by the orbit of the planets.
    ///
    /// Errors if the unit is not a rocket.
    pub fn rocket_travel_time_decrease(&self) -> Result<u32, Error> {
        self.ok_if_unit_type(Rocket)?;
        Ok(self.travel_time_decrease)
    }

    /// Whether the rocket can launch. It must not be used and it must
    /// currently be on a planet.
    ///
    /// Errors if the unit is not a rocket.
    pub(crate) fn can_launch_rocket(&self) -> Result<bool, Error> {
        match self.location {
            OnMap(_) => Ok(!self.rocket_is_used()?),
            _ => Ok(false),
        }
    }

    /// Updates the rocket as if it has launched by changing its location and
    /// marking it as used.
    ///
    /// Errors if the unit is not a rocket.
    pub(crate) fn launch_rocket(&mut self) -> Result<(), Error> {
        if self.can_launch_rocket()? {
            self.location = InSpace;
            self.is_used = true;
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Updates the rocket's location as if it has landed.
    ///
    /// Errors if the unit is not a rocket, or if it cannot be landed.
    pub(crate) fn land_rocket(&mut self, location: MapLocation) -> Result<(), Error> {
        if self.location == InSpace {
            self.ok_if_unit_type(Rocket)?;
            self.location = OnMap(location);
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    /// Boards a rocket. The unit must be ready to move.
    pub(crate) fn board_rocket(&mut self, rocket_id: UnitID) -> Result<(), Error> {
        if self.is_move_ready()? {
            self.movement_heat += self.movement_cooldown;
            self.location = InGarrison(rocket_id);
            Ok(())
        } else {
            Err(GameError::InvalidAction)?
        }
    }

    // ************************************************************************
    // **************************** OTHER METHODS *****************************
    // ************************************************************************

    /// Research the next level.
    pub(crate) fn research(&mut self) -> Result<(), Error> {
        match self.unit_type {
            Worker => match self.level {
                0 => { self.harvest_amount += 1; },
                1 => { self.build_health += 1; },
                2 => { self.build_health += 1; },
                3 => { self.build_health += 3; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Knight => match self.level {
                0 => { self.defense += 5; },
                1 => { self.defense += 5; },
                2 => { self.is_ability_unlocked = true; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Ranger => match self.level {
                0 => { self.movement_cooldown -= 5; },
                1 => { self.vision_range += 30; },
                2 => { self.is_ability_unlocked = true; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Mage => match self.level {
                0 => { self.damage += 15; },
                1 => { self.damage += 15; },
                2 => { self.damage += 15; },
                3 => { self.is_ability_unlocked = true; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Healer => match self.level {
                0 => { self.damage -= 2; },
                1 => { self.damage -= 5; },
                2 => { self.is_ability_unlocked = true; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Rocket => match self.level {
                // TODO: rocket unlocking
                0 => { self.is_ability_unlocked = true; },
                1 => { self.travel_time_decrease -= 20; },
                2 => { self.max_capacity += 4; },
                _ => Err(GameError::InvalidResearchLevel)?,
            },
            Factory => Err(GameError::InvalidResearchLevel)?,
        }
        self.level += 1;
        Ok(())
    }

    /// Process the end of the round.
    pub(crate) fn end_round(&mut self) {
        self.movement_heat -= cmp::min(HEAT_LOSS_PER_ROUND, self.movement_heat);
        self.attack_heat -= cmp::min(HEAT_LOSS_PER_ROUND, self.attack_heat);
        self.ability_heat -= cmp::min(HEAT_LOSS_PER_ROUND, self.ability_heat);
        self.countdown -= cmp::min(COUNTDOWN_PER_ROUND, self.countdown);
        self.has_worker_acted = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement() {
        let loc_a = MapLocation::new(Planet::Earth, 0, 0);
        let loc_b = MapLocation::new(Planet::Earth, 1, 1);
        let mut unit = Unit::new(1, Team::Red, Healer, 0, OnMap(loc_a)).unwrap();
        assert_eq!(unit.location(), OnMap(loc_a));
        assert_gt!(unit.movement_cooldown().unwrap(), 0);
        assert!(unit.is_move_ready().unwrap());
        assert_eq!(unit.location(), OnMap(loc_a));
        assert_eq!(unit.movement_heat().unwrap(), 0);

        // Move to a location, and fail to move immediately after.
        assert!(unit.move_to(loc_b).is_ok());
        assert!(!unit.is_move_ready().unwrap());
        assert!(unit.move_to(loc_a).is_err());
        assert_eq!(unit.location(), OnMap(loc_b));

        // Wait one round, and fail to move again.
        unit.end_round();
        assert!(unit.movement_heat().unwrap() > MAX_HEAT_TO_ACT);
        assert!(!unit.is_move_ready().unwrap());
        assert!(unit.move_to(loc_a).is_err());
        assert_eq!(unit.location(), OnMap(loc_b));

        // Wait one more round, and succesfully move.
        unit.end_round();
        assert!(unit.movement_heat().unwrap() < MAX_HEAT_TO_ACT);
        assert!(unit.is_move_ready().unwrap());
        assert!(unit.move_to(loc_a).is_ok());
        assert_eq!(unit.location(), OnMap(loc_a));
    }

    #[test]
    fn test_movement_error() {
        let loc = MapLocation::new(Planet::Earth, 0, 0);
        let adjacent_loc = MapLocation::new(Planet::Earth, 1, 0);

        let mut factory = Unit::new(1, Team::Red, Factory, 0, OnMap(loc)).unwrap();
        assert!(factory.movement_heat().is_err());
        assert!(factory.movement_cooldown().is_err());
        assert!(factory.is_move_ready().is_err());
        assert!(factory.move_to(adjacent_loc).is_err());

        let mut rocket = Unit::new(1, Team::Red, Rocket, 0, OnMap(loc)).unwrap();
        assert!(rocket.movement_heat().is_err());
        assert!(rocket.movement_cooldown().is_err());
        assert!(rocket.is_move_ready().is_err());
        assert!(rocket.move_to(adjacent_loc).is_err());
    }

    #[test]
    fn test_combat() {
    }

    #[test]
    fn test_special_abilities() {
        let loc = MapLocation::new(Planet::Earth, 0, 0); 

        // Worker and Rocket cannot use ability
        let worker = Unit::new(1, Team::Red, Worker, 0, OnMap(loc)).unwrap();
        assert!(worker.ok_if_ability().is_err());
        let rocket = Unit::new(1, Team::Red, Worker, 0, OnMap(loc)).unwrap();
        assert!(rocket.ok_if_ability().is_err());


        // Other units can use ability.
        let knight = Unit::new(1, Team::Red, Knight, 3, OnMap(loc)).unwrap();
        assert!(knight.ok_if_ability().is_ok());
        let ranger = Unit::new(1, Team::Red, Knight, 3, OnMap(loc)).unwrap();
        assert!(ranger.ok_if_ability().is_ok());
        let mage = Unit::new(1, Team::Red, Knight, 3, OnMap(loc)).unwrap();
        assert!(mage.ok_if_ability().is_ok());
        let healer = Unit::new(1, Team::Red, Knight, 3, OnMap(loc)).unwrap();
        assert!(healer.ok_if_ability().is_ok());

        // Unit cannot use ability when ability heat >= max heat to act 
        let mut ranger = Unit::new(1, Team::Red, Ranger, 3, OnMap(loc)).unwrap();
        ranger.ability_heat = MAX_HEAT_TO_ACT;
        assert!(!ranger.is_ability_ready().unwrap());
        ranger.ability_heat = MAX_HEAT_TO_ACT + 10;
        assert!(!ranger.is_ability_ready().unwrap());
    }

    #[test]
    fn test_knight() {
        let loc = MapLocation::new(Planet::Earth, 0, 0);

        // Javelin should fail if unit is not a knight
        let mut worker = Unit::new(1, Team::Red, Worker, 0, OnMap(loc)).unwrap();
        assert!(worker.ok_if_javelin().is_err());
        assert!(worker.javelin().is_err());
    }

    #[test]
    fn test_ranger() {
        let loc_a = MapLocation::new(Planet::Earth, 0, 0);
        let loc_b = MapLocation::new(Planet::Earth, 0, 1);

        // Sniping should fail if unit is not a ranger
        let worker = Unit::new(1, Team::Red, Worker, 0, OnMap(loc_a)).unwrap();
        assert!(worker.ok_if_snipe().is_err());

        // Begin sniping
        let mut ranger = Unit::new(1, Team::Red, Ranger, 3, OnMap(loc_a)).unwrap();
        assert!(ranger.ok_if_snipe().is_ok());
        assert!(ranger.begin_snipe(loc_b).is_ok());
        assert!(ranger.process_snipe().is_none());
        assert_eq!(ranger.ranger_target_location().unwrap().unwrap(), loc_b);

        // Ranger can begin sniping at anytime as long as ability heat < max heat to act
        assert!(ranger.begin_snipe(loc_b).is_ok());

        // Process sniping
        let rounds = 200;
        for _ in 0..rounds {
            ranger.end_round();
        }
        assert!(ranger.process_snipe().is_some());
    }

    #[test]
    fn test_mage() {
        let loc_a = MapLocation::new(Planet::Earth, 0, 0);
        let loc_b = MapLocation::new(Planet::Earth, 0, 1);

        // Blinking moves mage to new location 
        let mut mage = Unit::new(1, Team::Red, Mage, 4, OnMap(loc_a)).unwrap();
        mage.blink(loc_b);
        assert_eq!(mage.location(), OnMap(loc_b));
    }

    #[test]
    fn test_healer() {
        let loc = MapLocation::new(Planet::Earth, 0, 0);

        // Overcharging should fail if unit is not a healer
        let mut worker = Unit::new(1, Team::Red, Worker, 0, OnMap(loc)).unwrap();
        assert!(worker.ok_if_overcharge().is_err());
        assert!(worker.overcharge().is_err());

        // Healer canfnot overcharge if it has insufficient research level.
        let healer = Unit::new(1, Team::Red, Healer, 0, OnMap(loc)).unwrap();
        assert!(healer.ok_if_overcharge().is_err());

        // Healer can overcharge if it has unlocked ability.
        let mut healer = Unit::new(1, Team::Red, Healer, 3, OnMap(loc)).unwrap();
        assert!(healer.ok_if_overcharge().is_ok());
        assert!(healer.overcharge().is_ok());
    }

    #[test]
    fn test_factory() {
        // A worker cannot produce a robot.
        let loc = MapLocation::new(Planet::Earth, 0, 0);
        let worker = Unit::new(1, Team::Red, Worker, 0, OnMap(loc)).unwrap();
        assert_err!(worker.ok_if_can_produce_robot(Mage), GameError::InappropriateUnitType);

        // A factory cannot produce a structure, but it can produce a mage.
        let mut factory = Unit::new(1, Team::Red, Factory, 0, OnMap(loc)).unwrap();
        assert_eq!(factory.factory_rounds_left().unwrap(), None);
        assert_err!(factory.ok_if_can_produce_robot(Factory), GameError::InappropriateUnitType);
        assert_err!(factory.ok_if_can_produce_robot(Rocket), GameError::InappropriateUnitType);
        assert!(factory.ok_if_can_produce_robot(Mage).is_ok());

        // The factory cannot produce anything when it's already busy.
        factory.produce_robot(Mage);
        assert!(factory.ok_if_can_produce_robot(Mage).is_err());

        // After a few rounds, the factory can produce again.
        for _ in 0..FACTORY_NUM_ROUNDS - 1 {
            assert_eq!(factory.process_factory_round(), None);
            assert!(factory.ok_if_can_produce_robot(Mage).is_err());
        }
        assert_eq!(factory.process_factory_round(), Some(Mage));
        assert!(factory.ok_if_can_produce_robot(Mage).is_ok());

        // Fill the factory to its max capacity.
        for id in 0..factory.structure_max_capacity().expect("unit has a capacity") {
            assert!(factory.load(id as UnitID).is_ok());
        }

        // The factory can produce one more robot, but it won't go in its garrison.
        assert!(factory.ok_if_can_produce_robot(Mage).is_ok());
        factory.produce_robot(Mage);
        for _ in 0..FACTORY_NUM_ROUNDS * 2 {
            assert_eq!(factory.process_factory_round(), None);
            assert!(factory.ok_if_can_produce_robot(Mage).is_err());
        }

        // After unloading the units, the factory will work again.
        for id in 0..factory.structure_max_capacity().expect("unit has a capacity") {
            assert_eq!(factory.unload_unit().unwrap(), id as UnitID);
        }
        assert_eq!(factory.process_factory_round(), Some(Mage));
        assert!(factory.ok_if_can_produce_robot(Mage).is_ok());
    }

    #[test]
    fn test_rockets() {
        let loc = MapLocation::new(Planet::Earth, 0, 0);
        let adjacent_loc = loc.add(Direction::North);
        let mars_loc = MapLocation::new(Planet::Mars, 0, 0);

        let mut rocket = Unit::new(1, Team::Red, Rocket, 0, OnMap(loc)).unwrap();
        let mut robot = Unit::new(2, Team::Red, Mage, 0, OnMap(adjacent_loc)).unwrap();

        // Rocket accessor methods should fail on a robot.
        assert!(robot.structure_max_capacity().is_err());
        assert!(robot.rocket_is_used().is_err());
        assert!(robot.structure_garrison().is_err());
        assert!(robot.load(0).is_err());
        assert!(robot.can_launch_rocket().is_err());
        assert!(robot.launch_rocket().is_err());
        assert!(robot.land_rocket(loc).is_err());
        assert!(robot.ok_if_can_unload_unit().is_err());
        assert!(robot.unload_unit().is_err());

        // Check accessor methods on the rocket.
        assert!(rocket.structure_max_capacity().unwrap() > 0);
        assert!(!rocket.rocket_is_used().unwrap());
        assert_eq!(rocket.structure_garrison().unwrap().len(), 0);
        assert!(rocket.ok_if_can_load().is_ok());
        assert!(rocket.ok_if_can_unload_unit().is_err());
        assert!(rocket.can_launch_rocket().unwrap());

        // The rocket cannot land.
        assert!(rocket.land_rocket(mars_loc).is_err());

        // Load a unit and launch into space.
        assert!(rocket.load(robot.id()).is_ok());
        assert_eq!(rocket.structure_garrison().unwrap(), vec![robot.id()]);
        assert!(rocket.ok_if_can_unload_unit().is_ok());

        assert_eq!(rocket.launch_rocket().unwrap(), ());
        assert_eq!(rocket.location(), InSpace);
        assert!(rocket.rocket_is_used().unwrap());

        // Proceed a round, then land the rocket.
        robot.end_round();
        rocket.end_round();
        assert_eq!(rocket.land_rocket(mars_loc).unwrap(), ());
        assert_eq!(rocket.location(), OnMap(mars_loc));

        // Unload the unit.
        assert!(rocket.ok_if_can_unload_unit().is_ok());
        assert_eq!(rocket.unload_unit().unwrap(), robot.id());
        assert!(!rocket.ok_if_can_unload_unit().is_ok());

        // Load too many units
        for i in 0..rocket.structure_max_capacity().unwrap() {
            assert!(rocket.ok_if_can_load().is_ok(), "failed to load unit {}", i);
            assert!(rocket.load(0).is_ok());
        }
        assert!(rocket.ok_if_can_load().is_err());
        assert!(rocket.load(0).is_err());
    }

    #[test]
    fn test_research() {
        // Create a unit and check that its basic fields are correct.
        let loc = MapLocation::new(Planet::Earth, 0, 0);
        let mut unit_a = Unit::new(1, Team::Red, Worker, 0, OnMap(loc)).unwrap();
        assert_eq!(unit_a.id(), 1);
        assert_eq!(unit_a.team(), Team::Red);
        assert_eq!(unit_a.unit_type(), Worker);

        // Upgrade it twice and check its stats have been updated.
        assert_eq!(unit_a.research_level(), 0);
        assert_eq!(unit_a.worker_harvest_amount().unwrap(), 3);
        assert_eq!(unit_a.worker_build_health().unwrap(), 5);

        unit_a.research().unwrap();
        assert_eq!(unit_a.research_level(), 1);
        assert_eq!(unit_a.worker_harvest_amount().unwrap(), 4);
        assert_eq!(unit_a.worker_build_health().unwrap(), 5);

        unit_a.research().unwrap();
        assert_eq!(unit_a.research_level(), 2);
        assert_eq!(unit_a.worker_harvest_amount().unwrap(), 4);
        assert_eq!(unit_a.worker_build_health().unwrap(), 6);

        // Create a unit with a default level above 0, and check its stats.
        let unit_b = Unit::new(2, Team::Red, Worker, 2, OnMap(loc)).unwrap();
        assert_eq!(unit_b.research_level(), 2);
        assert_eq!(unit_b.worker_harvest_amount().unwrap(), 4);
        assert_eq!(unit_b.worker_build_health().unwrap(), 6);
    }
}
