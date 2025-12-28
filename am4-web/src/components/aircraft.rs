use crate::components::format_thousands;
use crate::components::search::MultiSelect;
use crate::db::Data;
use am4::aircraft::custom::{CustomAircraft, Modification, Modifier};
use am4::aircraft::db::{LENGTH_MAX, LENGTH_MEAN};
use am4::aircraft::Aircraft;
use am4::user::Settings;
use leptos::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ACSelection {
    /// Single aircraft variant (used when only 1 engine exists)
    Single(Aircraft, Modification),
    /// Header for multiple variants
    Header(Aircraft),
    /// Engine variant row
    Variant(Aircraft, Modification),
}

impl ACSelection {
    pub fn aircraft(&self) -> &Aircraft {
        match self {
            Self::Single(a, _) => a,
            Self::Header(a) => a,
            Self::Variant(a, _) => a,
        }
    }

    pub fn modification(&self) -> Option<&Modification> {
        match self {
            Self::Single(_, m) => Some(m),
            Self::Variant(_, m) => Some(m),
            Self::Header(_) => None,
        }
    }
}

/// Signal for the active aircraft selection, provided as context
pub type ActiveAircraftSignal = RwSignal<Option<ACSelection>>;

#[allow(non_snake_case)]
#[component]
pub fn ACSearch() -> impl IntoView {
    let database = expect_context::<StoredValue<Option<Data>>>();
    let settings = expect_context::<RwSignal<Settings>>();
    let selected = RwSignal::new(Vec::<ACSelection>::new());
    let active = RwSignal::new(None::<ACSelection>);

    let search = Callback::new(move |q: String| {
        database.with_value(|db| {
            db.as_ref()
                .unwrap()
                .aircrafts
                .suggest(&q)
                .unwrap_or_default()
                .into_iter()
                .flat_map(|s| {
                    let mut items = Vec::new();
                    let res = s.item;
                    if res.variants.len() == 1 {
                        let mut mods = res.modifiers.clone();
                        settings.with(|s| {
                            if s.default_4x {
                                mods.mods.insert(Modifier::FourX);
                            }
                            if s.default_speed_mod {
                                mods.mods.insert(Modifier::Speed);
                            }
                            if s.default_fuel_mod {
                                mods.mods.insert(Modifier::Fuel);
                            }
                            if s.default_co2_mod {
                                mods.mods.insert(Modifier::Co2);
                            }
                        });
                        items.push(ACSelection::Single(res.variants[0].clone(), mods));
                    } else {
                        items.push(ACSelection::Header(res.variants[0].clone()));
                        for variant in res.variants {
                            let mut mods = res.modifiers.clone();
                            mods.engine = variant.priority;
                            settings.with(|s| {
                                if s.default_4x {
                                    mods.mods.insert(Modifier::FourX);
                                }
                                if s.default_speed_mod {
                                    mods.mods.insert(Modifier::Speed);
                                }
                                if s.default_fuel_mod {
                                    mods.mods.insert(Modifier::Fuel);
                                }
                                if s.default_co2_mod {
                                    mods.mods.insert(Modifier::Co2);
                                }
                            });
                            items.push(ACSelection::Variant(variant, mods));
                        }
                    }
                    items
                })
                .collect()
        })
    });

    let render_option = move |sel: ACSelection,
                              _idx: usize,
                              suggestions: Memo<Vec<ACSelection>>,
                              highlight_idx: ReadSignal<usize>,
                              _select: Callback<ACSelection>,
                              _update: Callback<ACSelection>,
                              update_all: Callback<
        Box<dyn Fn(ACSelection) -> ACSelection + Send + Sync>,
    >| {
        let render_mods = move |ac: &Aircraft, mods: &Modification| {
            let active_mods = &mods.mods;
            let ename = &ac.ename;
            let prio = ac.priority;

            let modified = CustomAircraft::from_aircraft_and_modifiers(ac.clone(), mods.clone());
            let speed = modified.aircraft.speed as u32;
            let fuel = modified.aircraft.fuel;
            let co2 = modified.aircraft.co2;

            let make_btn = |m: Modifier, label: &'static str| {
                let is_active = active_mods.contains(&m);

                view! {
                    <button
                        class:active=is_active
                        on:click=move |ev| {
                            ev.stop_propagation();
                            ev.prevent_default();
                            let target_mod = m.clone();
                            let should_enable = !is_active;
                            update_all
                                .run(
                                    Box::new(move |item: ACSelection| {
                                        match item {
                                            ACSelection::Single(ac, mut mods) => {
                                                if should_enable {
                                                    mods.mods.insert(target_mod.clone());
                                                } else {
                                                    mods.mods.remove(&target_mod);
                                                }
                                                ACSelection::Single(ac, mods)
                                            }
                                            ACSelection::Variant(ac, mut mods) => {
                                                if should_enable {
                                                    mods.mods.insert(target_mod.clone());
                                                } else {
                                                    mods.mods.remove(&target_mod);
                                                }
                                                ACSelection::Variant(ac, mods)
                                            }
                                            ACSelection::Header(ac) => ACSelection::Header(ac),
                                        }
                                    }),
                                );
                        }
                        on:mousedown=move |ev| ev.prevent_default()
                    >
                        {label}
                    </button>
                }
            };

            view! {
                <div class="stats-row">
                    <span class="engine-info">
                        <span class="engine-prio">{prio.get()}</span>
                        <span class="engine-name">{ename.clone()}</span>
                    </span>
                    <span class="stats-inline">
                        {make_btn(Modifier::FourX, "X")}{make_btn(Modifier::Speed, "S")}
                        <span class="stat-val speed">{speed}</span> {make_btn(Modifier::Fuel, "F")}
                        <span class="stat-val fuel">{format!("{:.2}", fuel)}</span>
                        {make_btn(Modifier::Co2, "C")}
                        <span class="stat-val co2">{format!("{:.2}", co2)}</span>
                    </span>
                </div>
            }
        };

        match sel {
            ACSelection::Header(ref ac) => {
                let group_id = format!("ac-{}", u16::from(ac.id));
                let header_shortname = ac.shortname.clone();

                let is_variant_highlighted = move || {
                    let hi = highlight_idx.get();
                    suggestions.with(|suggs| {
                        if let Some(ACSelection::Variant(variant_ac, _)) = suggs.get(hi) {
                            return variant_ac.shortname == header_shortname;
                        }
                        false
                    })
                };

                view! {
                    <div
                        class="ac-option header"
                        data-group=group_id
                        class:variant-highlighted=is_variant_highlighted
                    >
                        <div class="main-row">
                            <img
                                class="ac-icon"
                                src=format!("/assets/img/aircraft/{}.webp", ac.img)
                            />
                            <div class="left">
                                <span class="name">
                                    {format!("{} {}", ac.manufacturer, ac.name)}
                                </span>
                                <span class="code">{"("}{ac.shortname.to_string()}{")"}</span>
                            </div>
                            <div class="right">
                                <span class="price">
                                    {format!("$ {}", format_thousands(ac.cost))}
                                </span>
                            </div>
                        </div>
                    </div>
                }
                .into_any()
            }
            ACSelection::Single(ref ac, ref mods) => view! {
                <div class="ac-option single">
                    <div class="main-row">
                        <img class="ac-icon" src=format!("/assets/img/aircraft/{}.webp", ac.img) />
                        <div class="left">
                            <span class="name">{format!("{} {}", ac.manufacturer, ac.name)}</span>
                            <span class="code">{"("}{ac.shortname.to_string()}{")"}</span>
                        </div>
                        <div class="right">
                            <span class="price">{format!("$ {}", format_thousands(ac.cost))}</span>
                        </div>
                    </div>
                    <div class="variant-row">{render_mods(ac, mods)}</div>
                </div>
            }
            .into_any(),
            ACSelection::Variant(ref ac, ref mods) => {
                let group_id = format!("ac-{}", u16::from(ac.id));
                view! {
                    <div class="ac-option variant" data-group=group_id>
                        <div class="variant-row">{render_mods(ac, mods)}</div>
                    </div>
                }
                .into_any()
            }
        }
    };

    let render_pill = move |sel: ACSelection, remove: Callback<()>| {
        let ac = sel.aircraft();
        let mods = sel.modification();

        let mod_str = if let Some(m) = mods {
            let mut s = String::new();
            if m.mods.contains(&Modifier::Speed) {
                s.push('s');
            }
            if m.mods.contains(&Modifier::Fuel) {
                s.push('f');
            }
            if m.mods.contains(&Modifier::Co2) {
                s.push('c');
            }
            if m.mods.contains(&Modifier::FourX) {
                s.push('x');
            }
            s
        } else {
            String::new()
        };

        let prio = match mods {
            Some(m) if m.engine.get() != 0 => m.engine.get().to_string(),
            _ => String::new(),
        };

        let suffix = if !prio.is_empty() || !mod_str.is_empty() {
            format!("[{}{}]", prio, mod_str)
        } else {
            String::new()
        };

        view! {
            <div class="ac-pill">
                <img src=format!("/assets/img/aircraft/{}.webp", ac.img) />
                <div class="pill-info">
                    <span class="code">{ac.shortname.to_string()}</span>
                    {(!suffix.is_empty()).then(|| view! { <span class="mods">{suffix}</span> })}
                </div>
                <span
                    class="remove"
                    on:click=move |ev| {
                        ev.stop_propagation();
                        remove.run(());
                    }
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                    >
                        <line x1="18" y1="6" x2="6" y2="18"></line>
                        <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                </span>
            </div>
        }
        .into_any()
    };

    provide_context(active);

    view! {
        <div id="ac-search">
            <label>"Aircraft"</label>
            <MultiSelect
                selected=selected
                active=active
                search=search
                max_items=1usize
                placeholder="Search aircraft..."
                render_option=render_option
                render_pill=render_pill
            />
        </div>
    }
}

/// Component that renders the aircraft details card
/// Must be rendered inside a component that has ACSearch as an ancestor
#[allow(non_snake_case)]
#[component]
pub fn ACDetails() -> impl IntoView {
    let active = expect_context::<ActiveAircraftSignal>();

    view! {
        {move || {
            active
                .get()
                .map(|sel| {
                    let ac = sel.aircraft().clone();
                    let mods = sel.modification().cloned().unwrap_or_default();
                    let custom = CustomAircraft::from_aircraft_and_modifiers(ac, mods);
                    view! { <Ac aircraft=custom.aircraft /> }
                })
        }}
    }
}

#[allow(non_snake_case)]
#[component]
fn Ac(aircraft: Aircraft) -> impl IntoView {
    let width = if aircraft.length == 0 {
        LENGTH_MEAN / LENGTH_MAX
    } else {
        aircraft.length as f32 / LENGTH_MAX
    } * 250f32;

    view! {
        <div class="ac-card">
            <h3>
                {aircraft.manufacturer} " " {aircraft.name.to_string()} " ("
                <code>{aircraft.shortname.to_string()}</code> ", " {aircraft.r#type.to_string()} ")"
            </h3>
            <table>
                <tr>
                    <th>{"Engine"}</th>
                    <td>
                        {aircraft.ename} " (id: " {aircraft.eid} ", rank: "
                        {format!("{}", aircraft.priority)} ")"
                    </td>
                </tr>
                <tr>
                    <th>{"Speed"}</th>
                    <td>{format!("{:.0}", aircraft.speed)} " km/h"</td>
                </tr>
                <tr>
                    <th>{"Fuel"}</th>
                    <td>{format!("{:.2}", aircraft.fuel)} " lbs/km"</td>
                </tr>
                <tr>
                    <th>{"CO2"}</th>
                    <td>{format!("{:.2}", aircraft.co2)} " kg/pax/km"</td>
                </tr>
                <tr>
                    <th>{"Cost"}</th>
                    <td>"$ " {format_thousands(aircraft.cost)}</td>
                </tr>
                <tr>
                    <th>{"Capacity"}</th>
                    <td>{aircraft.capacity}</td>
                </tr>
                <tr>
                    <th>{"Range"}</th>
                    <td>{format_thousands(aircraft.range as u32)} " km"</td>
                </tr>
                <tr>
                    <th>{"Runway"}</th>
                    <td>{format_thousands(aircraft.rwy as u32)} " ft"</td>
                </tr>
                <tr>
                    <th>{"Check cost"}</th>
                    <td>"$ " {format_thousands(aircraft.check_cost)}</td>
                </tr>
                <tr>
                    <th>{"Maintenance"}</th>
                    <td>{aircraft.maint} " hr"</td>
                </tr>
                <tr>
                    <th>{"Ceiling"}</th>
                    <td>{format_thousands(aircraft.ceil as u32)} " ft"</td>
                </tr>
                <tr>
                    <th>{"Personnel"}</th>
                    <td>
                        {format!(
                            "{} pilots, {} crew, {} engineers, {} technicians",
                            aircraft.pilots,
                            aircraft.crew,
                            aircraft.engineers,
                            aircraft.technicians,
                        )}

                    </td>
                </tr>
                <tr>
                    <th>{"Dimensions"}</th>
                    <td>{format!("{} m × {} m", aircraft.length, aircraft.wingspan)}</td>
                </tr>
            </table>
            <div id="ac-img">
                <img src=format!("/assets/img/aircraft/{}.webp", aircraft.img) width=width />
            </div>
        </div>
    }
}
