use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Spell {
    area: Option<SpellArea>,
    focus: Option<String>,
    effect: Option<Vec<String>>,
    component: String,
    target: SpellTarget,
}

#[derive(Debug, PartialEq)]
enum SpellArea {
    Breakpoint(SpellBreakpoint),
    MediaQuery(String),
}

impl FromStr for SpellArea {
    type Err = &'static str;

    fn from_str(area: &str) -> Result<Self, Self::Err> {
        match area.chars().next() {
            Some('(') => match area.find(')') {
                Some(i) => Ok(SpellArea::MediaQuery(String::from(&area[1..i]))),
                None => Err("missing ')' in spell area"),
            },
            None => Err("spell not long enough"),
            _ => Ok(SpellArea::Breakpoint(area.parse()?)),
        }
    }
}

#[derive(Debug, PartialEq)]
enum SpellBreakpoint {
    Small,
    Medium,
    Large,
    XLarge,
    XXLarge,
}

impl FromStr for SpellBreakpoint {
    type Err = &'static str;

    fn from_str(bp: &str) -> Result<Self, Self::Err> {
        Ok(match bp {
            "sm" => SpellBreakpoint::Small,
            "md" => SpellBreakpoint::Medium,
            "lg" => SpellBreakpoint::Large,
            "xl" => SpellBreakpoint::XLarge,
            "xxl" => SpellBreakpoint::XXLarge,
            _ => return Err("invalid breakpoint for area"),
        })
    }
}

#[derive(Debug, PartialEq)]
enum SpellTarget {
    CSSValue(String),
    Variables(Vec<String>),
}

impl FromStr for SpellTarget {
    type Err = &'static str;

    fn from_str(target: &str) -> Result<Self, Self::Err> {
        // TODO: will probably need a better check in real-world examples
        let is_variables = target.chars().any(|c| c == '_') && target.chars().all(|c| c.is_alphanumeric() || c == '_');
        if !is_variables {
            return Ok(Self::CSSValue(String::from(target)));
        }
        let variables: Vec<String> = target.split('_').map(String::from).collect();
        if variables.is_empty() { return Err("empty target!"); }
        Ok(Self::Variables(variables))
    }
}

impl FromStr for Spell {
    type Err = &'static str;

    fn from_str(spell: &str) -> Result<Self, Self::Err> {
        let area_end = spell.find("__");
        let area = match area_end {
            Some(i) => Some(spell[..i].parse()?),
            None => None,
        };

        let focus_start = match area_end {
            Some(i) => i + 2,
            None => 0,
        };
        let focus_len = match spell[focus_start..].chars().next() {
            Some('{') => match spell[focus_start..].find('}') {
                Some(i) => Some(i),
                None => return Err("spell ends without closing focus"),
            },
            None => return Err("spell ends too early while looking for focus"),
            _ => None,
        };

        let focus = match focus_len {
            Some(i) => Some(String::from(&spell[focus_start+1..focus_start+i])),
            None => None,
        };

        let focus_len = focus_len.unwrap_or(0);

        let effect_start = focus_start + focus_len + if focus_len != 0 { 1 } else { 0 };

        let effect = match spell[effect_start..].find([':', '=']) {
            Some(i) => match spell.chars().nth(effect_start + i) {
                Some(c) => match c {
                    '=' => None, // effect ends before component
                    ':' => Some(
                        spell[effect_start..effect_start+i].split(',').map(String::from).collect()
                    ),
                    c => unreachable!("impossible to match character {}", c),
                },
                None => unreachable!("spell is shorter than itself"),
            }
            None => None,
        };

        let component_start = match effect {
                Some(_) => effect_start + spell[effect_start..].find(':').unwrap() + 1,
                None => match focus {
                    Some(_) => focus_start + focus_len + 1,
                    None => focus_start,
                },
        };

        let component_len = match spell[component_start..].find('=') {
            Some(i) => i,
            None => return Err("expected '=' after component but could not find one"),
        };
        let component = String::from(&spell[component_start..component_start + component_len]);

        let target = spell[component_start + component_len + 1..].parse::<SpellTarget>()?;

        Ok(Self{
            area,
            focus,
            effect,
            component,
            target,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Spell, SpellArea, SpellBreakpoint, SpellTarget};

    fn expect(spell_str: &str, spell: Spell) {
        assert_eq!(spell_str.parse(), Ok(spell));
    }

    #[test]
    fn simple_spell() {
        expect("border-radius=8px", Spell{
            area: None,
            focus: None,
            effect: None,
            component: String::from("border-radius"),
            target: SpellTarget::CSSValue(String::from("8px")),
        });
    }

    #[test]
    fn spell_with_area() {
        expect("(width>=768px)__br=0.375rem", Spell {
            area: Some(SpellArea::MediaQuery(String::from("width>=768px"))),
            focus: None,
            effect: None,
            component: String::from("br"),
            target: SpellTarget::CSSValue(String::from("0.375rem")),
        });
    }

    #[test]
    fn spell_with_focus() {
        expect("{[hidden]_>_p:hover:active}color=red", Spell{
            area: None,
            focus: Some(String::from("[hidden]_>_p:hover:active")),
            effect: None,
            component: String::from("color"),
            target: SpellTarget::CSSValue(String::from("red")),
        });
    }

    #[test]
    fn spell_with_effect() {
        expect("hover,active:background-color=darkgrey", Spell{
            area: None,
            focus: None,
            effect: Some(vec![String::from("hover"), String::from("active")]),
            component: String::from("background-color"),
            target: SpellTarget::CSSValue(String::from("darkgrey")),
        });
    }

    #[test]
    fn spell_with_variables() {
        expect("btn=8px_lightgrey_grey_darkgrey", Spell {
            area: None,
            focus: None,
            effect: None,
            component: String::from("btn"),
            target: SpellTarget::Variables(vec![
                String::from("8px"),
                String::from("lightgrey"),
                String::from("grey"),
                String::from("darkgrey")
            ]),
        });
    }

    #[test]
    fn complex_spell_area_focus() {
        expect("md__{[hidden]_>_p:hover:active}color=red", Spell{
            area: Some(SpellArea::Breakpoint(SpellBreakpoint::Medium)),
            focus: Some(String::from("[hidden]_>_p:hover:active")),
            effect: None,
            component: String::from("color"),
            target: SpellTarget::CSSValue(String::from("red")),
        });
    }

    #[test]
    fn complex_spell_area_effect() {
        expect("md__hover,active:color=red", Spell{
            area: Some(SpellArea::Breakpoint(SpellBreakpoint::Medium)),
            focus: None,
            effect: Some(vec![String::from("hover"), String::from("active")]),
            component: String::from("color"),
            target: SpellTarget::CSSValue(String::from("red")),
        });
    }

    #[test]
    fn complex_spell_area_focus_effect() {
        expect("md__{_>_p}hover:display=none", Spell{
            area: Some(SpellArea::Breakpoint(SpellBreakpoint::Medium)),
            focus: Some(String::from("_>_p")),
            effect: Some(vec![String::from("hover")]),
            component: String::from("display"),
            target: SpellTarget::CSSValue(String::from("none")),
        });
    }
}
