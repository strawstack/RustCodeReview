use std::io;
use std::io::prelude::*;
use regex::{Regex};
use std::collections::HashMap;
use std::cmp::{max, min};

type GroupIndex = i32;
type EffectivePower = i32;
type Initiative = i32;
type Damage = i32;

const d : bool = false;
const d2 : bool = false;

#[allow(dead_code)]
#[derive(Debug)]
struct Attack {
    damage: i32,
    kind: i32 
}

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, PartialEq)]
#[derive(Hash)]
enum Army {
    Immune,
    Infection
}

#[allow(dead_code)]
#[derive(Debug)]
struct Group {
    id: i32,
    units: i32,
    health: i32,
    weak: i32,
    immune: i32,
    attack: Attack,
    init: i32,
    army: Army
}

#[allow(dead_code)]
#[derive(Debug)]
struct WeakImmune {
    weak: i32,
    immune: i32
}

impl Group {
    
    fn effective_power(&self) -> i32 {
        self.units * self.attack.damage
    }
    
    fn calculate_damage(&self, other : &Group) -> i32 {
        let damage = self.effective_power();
        if self.attack.kind & other.immune != 0 {
            0
        } else if self.attack.kind & other.weak != 0 {
            damage * 2
        } else {
            damage
        }
    }
    
    fn attack(&self, other : &mut Group) {
        let damage = self.calculate_damage(other);
        let units_lost = min(other.units, damage / other.health);
        let remaining_units = max(0, other.units - units_lost);
        if d {
            println!("{} group {} attacks defending group {}, killing {} units", 
                if self.army == Army::Immune {"Immune"} else {"Infection"}, self.id + 1, other.id + 1, units_lost);
        }
        other.units = remaining_units; 
    }
}

fn kind_lookup(kind : &str) -> i32 {
    match kind {
        "cold" => 1,
        "radiation" => 2,
        "slashing" => 4,
        "fire" => 8,
        "bludgeoning" => 16,
        _ => 0
    }
}

fn parse_line<'a>(line : &'a str, re : &Regex) -> Vec<&'a str> {
    let caps = re.captures(line).expect("Capture fails.");
    (1..caps.len())
        .map(|i| 
            caps.get(i).unwrap().as_str()
        )
        .collect()
}

fn parse_lines<'a>(lines : Vec<&'a str>) -> Vec<Vec<&'a str>> {
    let re_line = Regex::new(r"^(\d+).+?(\d+).+?(\d+) ([a-z]+).+?(\d+)$").expect("Regex fails.");
    let re_bracket_line = Regex::new(r"^(\d+).+?(\d+).+?\((.+?)\).+?(\d+) ([a-z]+).+?(\d+)$").expect("Regex fails.");
    lines
        .iter()
        .map(|line| 
            if line.contains("(") { 
                parse_line(line, &re_bracket_line) 
            } else { 
                parse_line(line, &re_line) 
            })
        .collect()
}

fn seperate_and_filter_lines<'a>(lines : &'a str) -> Vec<&'a str> {
    lines
        .split("\n")
        .filter(|line| line.chars().count() > 20)
        .collect()
}

fn parse_weak_or_immune(line : &str) -> i32 {
    let re = Regex::new(r"[, ]").expect("Regex fails.");
    let kinds : Vec<&str> = re.split(line).collect();
    kinds.iter()
        .fold(0, |acc, x| acc | kind_lookup(x) )
}

fn parse_weak_and_immune(line : &str) -> WeakImmune {
    if line.contains(";") {
        let sep : Vec<&str> = line.split("; ").collect();
        if sep[0].contains("weak") {
            WeakImmune {
                weak: parse_weak_or_immune(sep[0]),
                immune: parse_weak_or_immune(sep[1])
            }     
        } else {
            WeakImmune {
                weak: parse_weak_or_immune(sep[1]),
                immune: parse_weak_or_immune(sep[0])
            }     
        }
    } else {
        if line.contains("weak") {
            WeakImmune {
                weak: parse_weak_or_immune(line),
                immune: 0
            }     
        } else {
            WeakImmune {
                weak: 0,
                immune: parse_weak_or_immune(line)
            }     
        }   
    } 
}

fn line_to_group(index : i32, line : &Vec<&str>, army: Army) -> Group {
    Group {
        id: index,
        units: line[0].parse().expect("Parse fails."),
        health: line[1].parse().expect("Parse fails."),
        weak: 0,
        immune: 0, 
        attack: Attack {
            damage: line[2].parse().expect("Parse fails."),
            kind: kind_lookup(line[3])
        }, 
        init: line[4].parse().expect("Parse fails."),
        army: army
    } 
}

fn line_to_group_with_weak_immune(index : i32, line : &Vec<&str>, army : Army) -> Group {
    let weak_immune = parse_weak_and_immune(line[2]);
    Group {
        id: index,
        units: line[0].parse().expect("Parse fails."),
        health: line[1].parse().expect("Parse fails."),
        weak: weak_immune.weak,
        immune: weak_immune.immune, 
        attack: Attack {
            damage: line[3].parse().expect("Parse fails."),
            kind: kind_lookup(line[4])
        }, 
        init: line[5].parse().expect("Parse fails."),
        army: army
    } 
}

fn lines_to_groups(lines : Vec<Vec<&str>>, army : Army) -> Vec<Group> {
    lines.iter().enumerate()
        .map(|(i, line)|
            if line.len() == 5 { line_to_group(i as i32, line, army) } else { line_to_group_with_weak_immune(i as i32, line, army) }
        ).collect()
}

fn target_selection_phase(
    immune_groups : &Vec<Group>, 
    infection_groups : &Vec<Group>) 
    -> HashMap<(Army, GroupIndex), (Army, GroupIndex)> {
        let mut selection_order : Vec<(EffectivePower, Initiative, Army, GroupIndex)> = Vec::new();
        for (i, group) in immune_groups.iter().enumerate() {
            let power = group.effective_power();
            selection_order.push( (power, group.init, group.army, i as i32) );
        }
        for (i, group) in infection_groups.iter().enumerate() {
            let power = group.effective_power();
            selection_order.push( (power, group.init, group.army, i as i32) );
        }
        
        selection_order.sort_by(|a, b| (b.0, b.1).partial_cmp(&(a.0, a.1)).unwrap());
        
        let mut target_selections : HashMap<(Army, GroupIndex), (Army, GroupIndex)> = HashMap::new();
        
        // Each group completes a target selection
        for (_power, _init, army, group_index) in selection_order {
            let current_group : &Group = if army == Army::Immune { 
                    &immune_groups[group_index as usize] 
                } else { 
                    &infection_groups[group_index as usize] 
                };
            
            if current_group.units == 0 {
                continue;
            }            

            let other_army_name = if army == Army::Immune { Army::Infection } else { Army::Immune }; 

            let other_army : &Vec<Group> = if army == Army::Immune { infection_groups } else { immune_groups };
            let mut damage_numbers : Vec<(Damage, EffectivePower, Initiative, Army, GroupIndex)> = Vec::new();
            for (i, other_group) in other_army.iter().enumerate() {
                if other_group.units == 0 || target_selections.contains_key(&(other_army_name, i as i32)) {
                    continue;
                }
                let damage = current_group.calculate_damage(other_group);
                if d {
                    println!("{} group {} would deal defending group {} {} damage", 
                        if army == Army::Immune {"Immune"} else {"Infection"}, 
                        group_index + 1, i + 1, damage
                    );
                }
                damage_numbers.push((
                    damage,
                    other_group.effective_power(),
                    other_group.init,
                    other_group.army,
                    i as i32
                ));
            }
            damage_numbers.sort_by(|a, b| (b.0, b.1, b.2, b.4).partial_cmp(&(a.0, a.1, a.2, a.4)).unwrap() );
            if damage_numbers.len() == 0 {
                continue;    
            }
            let selected_group = damage_numbers[0]; 
            if selected_group.0 != 0 {
                target_selections.entry((selected_group.3, selected_group.4)).or_insert((army, group_index));   
            }
        }
        target_selections        
} 

fn execute_attack(
    army : Army, group_index : GroupIndex, 
    _target_army : Army, target_group_index : GroupIndex, 
    immune_groups : &mut Vec<Group>, infection_groups : &mut Vec<Group>) {
        if army == Army::Immune {
            immune_groups[group_index as usize].attack(&mut infection_groups[target_group_index as usize]);
        } else {
            infection_groups[group_index as usize].attack(&mut immune_groups[target_group_index as usize]);
        }
}
fn attack_phase(
    immune_groups : &mut Vec<Group>, 
    infection_groups : &mut Vec<Group>, 
    target_selections : &HashMap<(Army, GroupIndex), (Army, GroupIndex)>) {
       
        let mut reverse_target_selections : HashMap<(Army, GroupIndex), (Army, GroupIndex)> = HashMap::new();

        for (from, to) in target_selections {
            reverse_target_selections.insert(*to, *from);
        }
        if d {
            println!("");
            println!("Target Selections: {:?}", reverse_target_selections);
        }
        let mut attack_order : Vec<(Initiative, Army, GroupIndex)> = Vec::new();

        for (i, group) in immune_groups.iter().enumerate() {
            attack_order.push( (group.init, group.army, i as i32) );
        }
 
        for (i, group) in infection_groups.iter().enumerate() {
            attack_order.push( (group.init, group.army, i as i32) );
        }

        attack_order.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap() );
        
        if d {
            println!("");
            println!("{:?}", attack_order);
            println!("");
        }

        for (_init, army, group_index) in attack_order {
            match reverse_target_selections.get(&(army, group_index)) {
                Some((target_army, target_group_index)) => {
                    execute_attack(
                        army, group_index, 
                        *target_army, *target_group_index, 
                        immune_groups, infection_groups);
                },
                None => continue
            }
        }
}

fn end_condition(immune_groups : &Vec<Group>, infection_groups : &Vec<Group>) -> bool {
    let immune_count = immune_groups.iter().filter(|x| x.units > 0).collect::<Vec<&Group>>().len();
    let infection_count = infection_groups.iter().filter(|x| x.units > 0).collect::<Vec<&Group>>().len();
    immune_count == 0 || infection_count == 0
}

fn main() {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).expect("Read input fails.");

    let armies : Vec<&str> = buf.split("Infection").collect();

    let immune_lines : Vec<&str> = seperate_and_filter_lines(armies[0]);
    let immune_lines : Vec<Vec<&str>> = parse_lines(immune_lines);
    let mut immune_groups : Vec<Group> = lines_to_groups(immune_lines, Army::Immune);
    //println!("{:#?}", immune_groups);

    let infection_lines : Vec<&str> = seperate_and_filter_lines(armies[1]);
    let infection_lines : Vec<Vec<&str>> = parse_lines(infection_lines);
    let mut infection_groups : Vec<Group> = lines_to_groups(infection_lines, Army::Infection);
    //println!("{:#?}", infection_groups);
    
    if d2 {
        println!("");
        println!("Immune:");
        for (i, group) in immune_groups.iter().enumerate() {
            //println!("Group {} contains {} units", i + 1, group.units);
            println!("{:#?}", group);
        }
        println!("");
        println!("Infection:");
        for (i, group) in infection_groups.iter().enumerate() {
            //println!("Group {} contains {} units", i + 1, group.units);
            println!("{:#?}", group);
        }
        println!("");
    }

    loop {

        // Key receives damage from value
        let target_selections : HashMap<(Army, GroupIndex), (Army, GroupIndex)> = target_selection_phase(
            &immune_groups, 
            &infection_groups
        );

        attack_phase(&mut immune_groups, &mut infection_groups, &target_selections);

        if end_condition(&immune_groups, &infection_groups) {
            break;
        }        
    }

    let immune_count = immune_groups.iter().filter(|x| x.units > 0).collect::<Vec<&Group>>().len();
    let infection_count = infection_groups.iter().filter(|x| x.units > 0).collect::<Vec<&Group>>().len();
    if immune_count == 0 {
        println!("Final remaining units: {}", infection_groups.iter().fold(0, |acc, x| acc + x.units));
    } else {
        println!("Final remaining units: {}", immune_groups.iter().fold(0, |acc, x| acc + x.units));
    }
    println!("Complete.");
}
