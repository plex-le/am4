use crate::components::format_thousands;
use crate::db::Data;
use am4::aircraft::custom::CustomAircraft;
use am4::aircraft::AircraftType;
use am4::airport::Airport;
use am4::route::config::ConfigAlgorithm;
use am4::route::demand::{CargoDemand, PaxDemand};
use am4::route::metrics::ConfigVariant;
use am4::route::search::schedule::{
    CiStrategy, NumAircraftStrategy, ScheduleStrategy, SearchConfig, SortBy, TripsPerDay,
    TripsPerDayStrategy,
};
use am4::route::search::AbstractRoutes;
use am4::route::ticket::Ticket;
use am4::route::{Ci, Distance, FlightTime};
use am4::user::{AirportCodePref, GameMode, Settings};
use am4::utils::Filter;
use leptos::html::Select;
use leptos::prelude::*;
use std::num::NonZeroU8;

#[derive(Clone, Default, PartialEq, Copy)]
pub struct RouteStats {
    pub count: usize,
    pub time_ms: f64,
}

#[derive(Clone, PartialEq)]
pub struct WebScheduledRoute {
    pub origin: Airport,
    pub destination: Airport,
    pub stopover: Option<Airport>,
    pub direct_distance: Distance,
    pub total_distance: Distance,
    pub flight_time: FlightTime,
    pub ci: Ci,
    pub contribution: f32,
    pub trips_per_day: u8,
    pub num_aircraft: u8,
    pub config: ConfigVariant,
    pub ticket: Ticket,
    pub revenue: f32,
    pub fuel: f32,
    pub co2: f32,
    pub acheck_cost: f32,
    pub repair_cost: f32,
    pub profit: f32,
    pub demand: PaxDemand,
}

#[component]
fn FilterInput(
    label: &'static str,
    min_val: RwSignal<String>,
    max_val: RwSignal<String>,
    error: ReadSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <label>
            {label} <div class="range-inputs">
                <input
                    type="text"
                    placeholder="Min"
                    prop:value=min_val
                    class:invalid=move || error.get().is_some()
                    on:input=move |ev| min_val.set(event_target_value(&ev))
                />
                <span class="sep">"-"</span>
                <input
                    type="text"
                    placeholder="Max"
                    prop:value=max_val
                    class:invalid=move || error.get().is_some()
                    on:input=move |ev| max_val.set(event_target_value(&ev))
                />
            </div> {move || { error.get().map(|e| view! { <span class="input-error">{e}</span> }) }}
        </label>
    }
}

#[component]
fn StrategyInput(
    label: &'static str,
    value: RwSignal<String>,
    is_active: RwSignal<bool>,
    active_label: &'static str,
    error: ReadSignal<Option<String>>,
    #[prop(optional)] disable_button: Option<Signal<bool>>,
) -> impl IntoView {
    view! {
        <label>
            {label} <div class="input-with-btn">
                <input
                    type="number"
                    prop:value=value
                    class:invalid=move || error.get().is_some()
                    on:input=move |ev| {
                        let v = event_target_value(&ev);
                        value.set(v.clone());
                        if v.trim().is_empty() {
                            is_active.set(true);
                        } else {
                            is_active.set(false);
                        }
                    }
                />
                <button
                    class:active=move || is_active.get() && disable_button.is_none_or(|s| !s.get())
                    class:disabled=move || disable_button.is_some_and(|s| s.get())
                    prop:disabled=move || disable_button.is_some_and(|s| s.get())
                    on:click=move |_| {
                        if disable_button.is_none_or(|s| !s.get()) {
                            is_active.set(true);
                            value.set(String::new());
                        }
                    }
                >
                    {active_label}
                </button>
            </div> {move || { error.get().map(|e| view! { <span class="input-error">{e}</span> }) }}
        </label>
    }
}

#[component]
fn ConfigAlgoInput(
    algo: RwSignal<ConfigAlgorithm>,
    ac_type: Memo<Option<AircraftType>>,
) -> impl IntoView {
    let select_ref = NodeRef::<Select>::new();

    Effect::new(move |_| {
        let a = algo.get();
        if let Some(el) = select_ref.get() {
            el.set_value(&a.to_string());
        }
    });

    view! {
        <label class:disabled=move || {
            ac_type.get().is_none()
        }>
            "Config Algorithm"
            <select
                node_ref=select_ref
                class:disabled=move || ac_type.get().is_none()
                prop:disabled=move || ac_type.get().is_none()
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    if let Ok(alg) = val.parse() {
                        algo.set(alg);
                    }
                }
            >
                <option value="auto">"Auto"</option>
                {move || {
                    ac_type
                        .get()
                        .map(|t| match t {
                            AircraftType::Pax | AircraftType::Vip => {
                                view! {
                                    <option value="fjy">"F>J>Y"</option>
                                    <option value="fyj">"F>Y>J"</option>
                                    <option value="jfy">"J>F>Y"</option>
                                    <option value="jyf">"J>Y>F"</option>
                                    <option value="yfj">"Y>F>J"</option>
                                    <option value="yjf">"Y>J>F"</option>
                                    <option value="y">"Y Only"</option>
                                    <option value="j">"J Only"</option>
                                    <option value="f">"F Only"</option>
                                }
                                    .into_any()
                            }
                            AircraftType::Cargo => {
                                view! {
                                    <option value="lh">"L>H"</option>
                                    <option value="hl">"H>L"</option>
                                    <option value="l">"L Only"</option>
                                    <option value="h">"H Only"</option>
                                }
                                    .into_any()
                            }
                        })
                }}
                <option value="spread">"Spread"</option>
            </select>
        </label>
    }
}

#[component]
pub fn RouteOptions(
    #[prop(into)] ac_selected: RwSignal<Vec<CustomAircraft>>,
    #[prop(into)] ap_selected: RwSignal<Vec<Airport>>,
    #[prop(into)] set_routes: WriteSignal<Vec<WebScheduledRoute>>,
    #[prop(into)] set_stats: WriteSignal<RouteStats>,
) -> impl IntoView {
    let database = expect_context::<StoredValue<Option<Data>>>();
    let settings = expect_context::<RwSignal<Settings>>();
    let game_mode = expect_context::<RwSignal<GameMode>>();

    let dist_min = RwSignal::new(String::new());
    let dist_max = RwSignal::new(String::new());
    let (dist_error, set_dist_error) = signal(None::<String>);

    let ft_min = RwSignal::new(String::new());
    let ft_max = RwSignal::new(String::new());
    let (ft_error, set_ft_error) = signal(None::<String>);

    let sort_by = RwSignal::new(SortBy::ProfitPerAcPerDay);

    let num_ac_input = RwSignal::new(String::from("1"));
    let num_ac_max = RwSignal::new(false);
    let (num_ac_error, set_num_ac_error) = signal(None::<String>);

    let tpd_input = RwSignal::new(String::new());
    let tpd_max = RwSignal::new(true);
    let (tpd_error, set_tpd_error) = signal(None::<String>);

    let ci_input = RwSignal::new(String::from("200"));
    let ci_auto = RwSignal::new(false);
    let (ci_error, set_ci_error) = signal(None::<String>);

    let stopover_mode = RwSignal::new(false);
    let config_algo = RwSignal::new(ConfigAlgorithm::Auto);

    let ac_type = Memo::new(move |_| ac_selected.get().first().map(|c| c.aircraft.r#type.clone()));

    // Disable stopover if no max distance
    Effect::new(move |_| {
        let max = dist_max.get();
        if max.trim().is_empty() && stopover_mode.get_untracked() {
            stopover_mode.set(false);
        }
    });

    Effect::new(move |_| {
        let ac_sel = ac_selected.get();
        let ap_sel = ap_selected.get();
        let user_settings = settings.get();
        let gm = game_mode.get();
        let sort = sort_by.get();
        let d_min = dist_min.get();
        let d_max = dist_max.get();
        let f_min = ft_min.get();
        let f_max = ft_max.get();
        let n_ac = num_ac_input.get();
        let n_ac_is_max = num_ac_max.get();
        let tpd = tpd_input.get();
        let tpd_is_max = tpd_max.get();
        let ci_s = ci_input.get();
        let ci_is_auto = ci_auto.get();
        let stopover = stopover_mode.get();
        let algo = config_algo.get();

        if ac_sel.is_empty() || ap_sel.is_empty() {
            set_routes.set(vec![]);
            set_stats.set(RouteStats::default());
            set_dist_error.set(None);
            set_ft_error.set(None);
            set_num_ac_error.set(None);
            set_tpd_error.set(None);
            set_ci_error.set(None);
            return;
        }

        // Apply modifications
        let custom_ac = ac_sel[0].effective();

        let parse_range = |min: String, max: String, error_sig: &WriteSignal<Option<String>>| {
            let parse_val = |s: &str| -> Result<Option<f32>, String> {
                if s.trim().is_empty() {
                    return Ok(None);
                }
                s.parse::<f32>()
                    .map(Some)
                    .map_err(|_| "Invalid number".to_string())
            };

            let min_res = parse_val(&min);
            let max_res = parse_val(&max);

            match (min_res, max_res) {
                (Err(e), _) | (_, Err(e)) => {
                    error_sig.set(Some(e));
                    None
                }
                (Ok(l), Ok(r)) => {
                    if let (Some(min), Some(max)) = (l, r) {
                        if min < 0. || max < 0. {
                            error_sig.set(Some("Must be positive".into()));
                            return None;
                        }
                        if min > max {
                            error_sig.set(Some("Min > Max".into()));
                            return None;
                        }
                    }
                    error_sig.set(None);
                    Some((l, r))
                }
            }
        };

        let dist_filter = if let Some((l, r)) = parse_range(d_min, d_max, &set_dist_error) {
            match (l, r) {
                (Some(min), Some(max)) => {
                    Filter::Range(Distance::new_unchecked(min)..Distance::new_unchecked(max))
                }
                (Some(min), None) => Filter::RangeFrom(Distance::new_unchecked(min)..),
                (None, Some(max)) => Filter::RangeTo(..Distance::new_unchecked(max)),
                (None, None) => Filter::RangeFull,
            }
        } else {
            return; // Validation failed
        };

        let ft_max_specified = !f_max.trim().is_empty();
        let ft_filter = if let Some((l, r)) = parse_range(f_min, f_max, &set_ft_error) {
            match (l, r) {
                (Some(min), Some(max)) => {
                    Filter::Range(FlightTime::new_unchecked(min)..FlightTime::new_unchecked(max))
                }
                (Some(min), None) => Filter::RangeFrom(FlightTime::new_unchecked(min)..),
                (None, Some(max)) => Filter::RangeTo(..FlightTime::new_unchecked(max)),
                (None, None) => Filter::RangeFull,
            }
        } else {
            return; // Validation failed
        };

        let num_ac_strat = if n_ac_is_max {
            set_num_ac_error.set(None);
            NumAircraftStrategy::Maximise
        } else if n_ac.trim().is_empty() {
            set_num_ac_error.set(Some("Required".into()));
            return;
        } else {
            match n_ac.parse::<u8>().ok().and_then(NonZeroU8::new) {
                Some(n) => {
                    set_num_ac_error.set(None);
                    NumAircraftStrategy::Strict(n)
                }
                None => {
                    set_num_ac_error.set(Some("Must be > 0".into()));
                    return;
                }
            }
        };

        let tpd_strat = if tpd_is_max {
            set_tpd_error.set(None);
            TripsPerDayStrategy::Maximise
        } else if tpd.trim().is_empty() {
            set_tpd_error.set(Some("Required".into()));
            return;
        } else {
            match tpd.parse::<u8>().ok().and_then(TripsPerDay::new) {
                Some(t) => {
                    set_tpd_error.set(None);
                    TripsPerDayStrategy::Strict(t)
                }
                None => {
                    set_tpd_error.set(Some("Must be > 0".into()));
                    return;
                }
            }
        };

        let ci_strat = if ci_is_auto && ft_max_specified {
            set_ci_error.set(None);
            CiStrategy::AlignConstraint
        } else if ci_s.trim().is_empty() {
            set_ci_error.set(Some("Required".into()));
            return;
        } else {
            match ci_s.parse::<u8>().ok().and_then(|v| Ci::new(v).ok()) {
                Some(c) => {
                    set_ci_error.set(None);
                    CiStrategy::Strict(c)
                }
                None => {
                    set_ci_error.set(Some("0-200".into()));
                    return;
                }
            }
        };

        let performance = web_sys::window().unwrap().performance().unwrap();
        let start_time = performance.now();

        database.with_value(|db| {
            let data = db.as_ref().unwrap();
            if let Some(demands) = &data.demands {
                let search_config = SearchConfig {
                    user_settings: &user_settings,
                    distance_filter: dist_filter,
                    flight_time_filter: ft_filter,
                    schedule: ScheduleStrategy {
                        trips_per_day: tpd_strat,
                        num_aircraft: num_ac_strat,
                    },
                    ci: ci_strat,
                    sort_by: sort.clone(),
                    inflate_distance_with_stopover: stopover,
                    config: algo,
                };

                let mut all_results = Vec::new();

                for origin in ap_sel.iter() {
                    let abstract_routes = AbstractRoutes::new(
                        &data.airports,
                        &data.distances,
                        origin,
                        data.airports.data(),
                    );

                    let concrete = abstract_routes.with_aircraft(&custom_ac, &gm);
                    let scheduled = concrete.schedule(demands, &data.distances, &search_config);

                    for r in scheduled.routes() {
                        all_results.push(WebScheduledRoute {
                            origin: origin.clone(),
                            destination: r.destination.clone(),
                            stopover: r.stopover.as_ref().map(|s| s.0.clone()),
                            direct_distance: r.direct_distance,
                            total_distance: r.total_distance,
                            flight_time: r.flight_time,
                            ci: r.ci,
                            contribution: r.contribution,
                            trips_per_day: r.trips_per_day.get(),
                            num_aircraft: r.num_aircraft.get(),
                            config: r.config,
                            ticket: r.ticket,
                            revenue: r.revenue,
                            fuel: r.fuel,
                            co2: r.co2,
                            acheck_cost: r.acheck_cost,
                            repair_cost: r.repair_cost,
                            profit: r.profit,
                            demand: demands[(origin.idx, r.destination.idx)],
                        });
                    }
                }

                match sort {
                    SortBy::ProfitPerTrip => {
                        all_results.sort_by(|a, b| b.profit.partial_cmp(&a.profit).unwrap());
                    }
                    SortBy::ProfitPerAcPerDay => {
                        all_results.sort_by(|a, b| {
                            let pa = a.profit * a.trips_per_day as f32;
                            let pb = b.profit * b.trips_per_day as f32;
                            pb.partial_cmp(&pa).unwrap()
                        });
                    }
                }

                set_stats.set(RouteStats {
                    count: all_results.len(),
                    time_ms: performance.now() - start_time,
                });
                set_routes.set(all_results);
            }
        });
    });

    view! {
        <div class="route-options">
            <FilterInput label="Distance (km)" min_val=dist_min max_val=dist_max error=dist_error />
            <FilterInput label="Time (hr)" min_val=ft_min max_val=ft_max error=ft_error />
            <StrategyInput
                label="AC per route"
                value=num_ac_input
                is_active=num_ac_max
                active_label="Maximise"
                error=num_ac_error
            />
            <StrategyInput
                label="Trips per day per AC"
                value=tpd_input
                is_active=tpd_max
                active_label="Maximise"
                error=tpd_error
            />
            <StrategyInput
                label="Cost Index"
                value=ci_input
                is_active=ci_auto
                active_label="Align Max Time"
                error=ci_error
                disable_button=Signal::derive(move || { ft_max.get().trim().is_empty() })
            />
            <label class:disabled=move || {
                dist_max.get().trim().is_empty()
            }>
                "Inflate distance"
                <input
                    type="checkbox"
                    prop:checked=stopover_mode
                    prop:disabled=move || dist_max.get().trim().is_empty()
                    on:change=move |ev| stopover_mode.set(event_target_checked(&ev))
                />
            </label>
            <ConfigAlgoInput algo=config_algo ac_type=ac_type />
            <div class="toggle-group">
                <span>"Sort"</span>
                <div class="toggle-options">
                    <button
                        class:active=move || sort_by.get() == SortBy::ProfitPerAcPerDay
                        on:click=move |_| sort_by.set(SortBy::ProfitPerAcPerDay)
                    >
                        "$/d/ac"
                    </button>
                    <button
                        class:active=move || sort_by.get() == SortBy::ProfitPerTrip
                        on:click=move |_| sort_by.set(SortBy::ProfitPerTrip)
                    >
                        "$/t"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn RouteList(
    #[prop(into)] routes: ReadSignal<Vec<WebScheduledRoute>>,
    #[prop(into)] stats: ReadSignal<RouteStats>,
    #[prop(into)] show_origin: Signal<bool>,
) -> impl IntoView {
    let page = RwSignal::new(0usize);
    let page_size = 10usize;

    Effect::new(move |_| {
        routes.track();
        page.set(0);
    });

    let paged_results = Memo::new(move |_| {
        let all = routes.get();
        let p = page.get();
        let start = p * page_size;
        if start >= all.len() {
            return vec![];
        }
        let end = (start + page_size).min(all.len());
        all[start..end].to_vec()
    });

    let total_pages = Memo::new(move |_| {
        let s = stats.get();
        if s.count == 0 {
            1
        } else {
            s.count.div_ceil(page_size)
        }
    });

    view! {
        <Show when=move || {
            let s = stats.get();
            s.count > 0
        }>
            <div class="results-meta">
                <span>
                    {move || stats.get().count} " routes found in "
                    {move || format!("~{:.1}", stats.get().time_ms)} "ms"
                </span>
                <div class="pagination">
                    <button
                        disabled=move || page.get() == 0
                        on:click=move |_| page.update(|p| *p -= 1)
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path d="m15 18-6-6 6-6" />
                        </svg>
                    </button>
                    <span>{move || page.get() + 1} " / " {total_pages}</span>
                    <button
                        disabled=move || { page.get() + 1 == total_pages.get() }
                        on:click=move |_| page.update(|p| *p += 1)
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path d="m9 18 6-6-6-6" />
                        </svg>
                    </button>
                </div>
            </div>
        </Show>

        <div class="results-list">
            <For
                each=move || paged_results.get()
                key=|_| uuid::Uuid::new_v4()
                children=move |route| {
                    view! { <RouteCard route=route show_origin=show_origin /> }
                }
            />
        </div>
    }
}

#[derive(Clone)]
struct StatItem {
    label: &'static str,
    val: String,
    class: &'static str,
}

#[component]
pub fn RouteCard(route: WebScheduledRoute, show_origin: Signal<bool>) -> impl IntoView {
    let settings = expect_context::<RwSignal<Settings>>();
    let fmt_money = |v: f32| format!("$ {}", format_thousands(v));
    let fmt_dist = |d: f32| format!("{} km", format_thousands(d));

    let is_cargo = matches!(route.config, ConfigVariant::Cargo(_));

    let total_seconds = (route.flight_time.get() * 3600.0).round() as u64;
    let h = total_seconds / 3600;
    let m = (total_seconds % 3600) / 60;
    let s = total_seconds % 60;
    let ft_str = format!("{:02}:{:02}:{:02}", h, m, s);

    let dist_diff_pct = (route.total_distance.get() - route.direct_distance.get())
        / route.direct_distance.get()
        * 100.0;
    let dist_pct_str = if dist_diff_pct > 0.1 {
        format!(", +{:.1}%", dist_diff_pct)
    } else {
        String::new()
    };

    let format_code = move |ap: &Airport| {
        settings.with(|s| match s.airport_code_pref {
            AirportCodePref::Iata => ap.iata.to_string(),
            AirportCodePref::Icao => ap.icao.to_string(),
        })
    };

    let origin_code = format_code(&route.origin);
    let dest_code = format_code(&route.destination);

    let render_row = |label: &'static str, items: Vec<StatItem>| {
        view! {
            <span class="label">{label}</span>
            <div class="stat-group">
                {items
                    .into_iter()
                    .map(|item| {
                        view! {
                            <div class="stat-pair">
                                <span class=format!("letter {}", item.class)>{item.label}</span>
                                <span class="val">{item.val}</span>
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        }
    };

    let demand_stats = if is_cargo {
        let d: CargoDemand = (&route.demand).into();
        vec![
            StatItem {
                label: "L",
                val: format_thousands(d.l),
                class: "l",
            },
            StatItem {
                label: "H",
                val: format_thousands(d.h),
                class: "h",
            },
        ]
    } else {
        vec![
            StatItem {
                label: "Y",
                val: format_thousands(route.demand.y),
                class: "y",
            },
            StatItem {
                label: "J",
                val: format_thousands(route.demand.j),
                class: "j",
            },
            StatItem {
                label: "F",
                val: format_thousands(route.demand.f),
                class: "f",
            },
        ]
    };

    let contribution_str = format!("C${:.0}/f", route.contribution);

    let config_stats = match route.config {
        ConfigVariant::Cargo(c) => vec![
            StatItem {
                label: "L",
                val: format!("{}%", c.l),
                class: "l",
            },
            StatItem {
                label: "H",
                val: format!("{}%", c.h),
                class: "h",
            },
        ],
        ConfigVariant::Pax(c) => vec![
            StatItem {
                label: "Y",
                val: c.y.to_string(),
                class: "y",
            },
            StatItem {
                label: "J",
                val: c.j.to_string(),
                class: "j",
            },
            StatItem {
                label: "F",
                val: c.f.to_string(),
                class: "f",
            },
        ],
    };

    let ticket_stats = match route.ticket {
        Ticket::Cargo(t) => vec![
            StatItem {
                label: "L",
                val: format!("${:.2}", t.l),
                class: "l",
            },
            StatItem {
                label: "H",
                val: format!("${:.2}", t.h),
                class: "h",
            },
        ],
        Ticket::Pax(t) | Ticket::VIP(t) => vec![
            StatItem {
                label: "Y",
                val: format_thousands(t.y),
                class: "y",
            },
            StatItem {
                label: "J",
                val: format_thousands(t.j),
                class: "j",
            },
            StatItem {
                label: "F",
                val: format_thousands(t.f),
                class: "f",
            },
        ],
    };

    let max_tpd = (24.0 / route.flight_time.get()).floor() as u8;
    let tpd_warning = route.trips_per_day > max_tpd;

    let revenue_str = fmt_money(route.revenue);
    let fuel_price = settings.get().fuel_price.get();
    let co2_price = settings.get().co2_price.get();
    let fuel_cost = route.fuel * fuel_price / 1000.0;
    let co2_cost = route.co2 * co2_price / 1000.0;
    let fuel_cost_str = fmt_money(fuel_cost);
    let co2_cost_str = fmt_money(co2_cost);
    let fuel_amount_str = format_thousands(route.fuel);
    let co2_amount_str = format_thousands(route.co2);
    let acheck_cost_str = fmt_money(route.acheck_cost);
    let repair_cost_str = fmt_money(route.repair_cost);

    view! {
        <div class="route-card">
            <div class="header">
                <Show when=move || show_origin.get()>
                    <div class="dest-block">
                        <span class="label">"from"</span>
                        <div class="codes">
                            <span class="main">{origin_code.clone()}</span>
                        </div>
                        <div class="name">
                            {route.origin.name.to_string()} ", " {route.origin.country.to_string()}
                        </div>
                    </div>
                </Show>
                {route
                    .stopover
                    .map(|stop| {
                        let stop_code = format_code(&stop);
                        view! {
                            <div class="dest-block">
                                <span class="label">"via"</span>
                                <div class="codes">
                                    <span class="main">{stop_code}</span>
                                </div>
                                <div class="name">
                                    {stop.name.to_string()} ", " {stop.country.to_string()}
                                </div>
                            </div>
                        }
                    })}
                <div class="dest-block">
                    <span class="label">"to"</span>
                    <div class="codes">
                        <span class="main">{dest_code}</span>
                    </div>
                    <div class="name">
                        {route.destination.name.to_string()} ", "
                        {route.destination.country.to_string()}
                    </div>
                </div>
            </div>

            <div class=format!(
                "grid-info {}",
                if is_cargo { "cargo" } else { "" },
            )>
                {render_row("Demand", demand_stats)} {render_row("Config", config_stats)}
                {render_row("Tickets", ticket_stats)}
            </div>

            <div class="details-text">
                {fmt_dist(route.total_distance.get())} " (" {ft_str} {dist_pct_str} ")" " ⋅ CI="
                {route.ci.get()} " ⋅ " {contribution_str.clone()} <br /> {route.trips_per_day}
                " t/d/ac" " × " {route.num_aircraft} " ac" " ⋅ " {fmt_money(route.profit)} "/t"
                " ⋅ " {fmt_money(route.profit * route.trips_per_day as f32)} "/d/ac"
            </div>
            <Show when=move || tpd_warning>
                <div class="warning-text">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"></path>
                        <path d="M12 9v4"></path>
                        <path d="M12 17h.01"></path>
                    </svg>
                    <span>{format!("Exceeds 24hrs (max: {} t/d)", max_tpd)}</span>
                </div>
            </Show>

            <details class="cost-breakdown">
                <summary>
                    <svg
                        class="expand-arrow"
                        xmlns="http://www.w3.org/2000/svg"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path d="m6 9 6 6 6-6" />
                    </svg>
                </summary>
                <div class="cost-grid">
                    <span class="cost-label">"Revenue"</span>
                    <span class="cost-value revenue">{revenue_str}</span>
                    <span class="cost-label">"Fuel (" {fuel_amount_str.clone()} " lbs)"</span>
                    <span class="cost-value expense">{fuel_cost_str}</span>
                    <span class="cost-label">"CO₂ (" {co2_amount_str.clone()} " quota)"</span>
                    <span class="cost-value expense">{co2_cost_str}</span>
                    <span class="cost-label">"A-check"</span>
                    <span class="cost-value expense">{acheck_cost_str}</span>
                    <span class="cost-label">"Repair"</span>
                    <span class="cost-value expense">{repair_cost_str}</span>
                </div>
            </details>
        </div>
    }
}
