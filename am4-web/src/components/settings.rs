use crate::console::UserLogger;
use am4::user::{
    AircraftLoad, AirportCodePref, Co2Price, Co2Training, FuelPrice, FuelTraining, GameMode,
    HeavyTraining, LargeTraining, RepairTraining, RevenueLossTol, Settings, ValidationError,
    WearTraining,
};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

#[component]
fn ValidatedInput<F>(
    value: impl Fn() -> String + 'static + Send + Sync,
    update: F,
    label: &'static str,
    #[prop(optional)] min: Option<&'static str>,
    #[prop(optional)] max: Option<&'static str>,
    #[prop(optional)] step: Option<&'static str>,
) -> impl IntoView
where
    F: Fn(&mut Settings, String) -> Result<(), String> + 'static + Clone + Copy,
{
    let settings = expect_context::<RwSignal<Settings>>();
    let logger = expect_context::<UserLogger>();
    let (error_msg, set_error) = signal(None::<String>);

    view! {
        <label>
            {label}
            <input
                type="number"
                min=min
                max=max
                step=step
                class:invalid=move || error_msg.get().is_some()
                prop:value=value
                on:input=move |_| {
                    set_error.set(None);
                }
                on:change=move |ev| {
                    let target = ev.target().unwrap();
                    let val = target.unchecked_into::<HtmlInputElement>().value();
                    settings
                        .update(|s| {
                            match update(s, val.clone()) {
                                Ok(_) => {
                                    set_error.set(None);
                                    logger
                                        .info(
                                            format!(
                                                "updated setting '{}': {}",
                                                label.to_lowercase(),
                                                val,
                                            ),
                                        );
                                }
                                Err(e) => {
                                    set_error.set(Some(e));
                                }
                            }
                        });
                }
            /> {move || { error_msg.get().map(|e| view! { <span class="input-error">{e}</span> }) }}
        </label>
    }
}

#[component]
pub fn SettingsPanel() -> impl IntoView {
    let settings = expect_context::<RwSignal<Settings>>();
    let game_mode = expect_context::<RwSignal<GameMode>>();
    let logger = expect_context::<UserLogger>();

    view! {
        <div class="settings-panel">
            <details>
                <summary>"User Settings"</summary>
                <div class="settings-grid">
                    <div class="setting-group">
                        <h4>"General"</h4>
                        <div class="mode-toggle">
                            <span>"Game Mode"</span>
                            <div class="toggle-options">
                                <button
                                    class:active=move || game_mode.get() == GameMode::Easy
                                    on:click=move |_| {
                                        game_mode.set(GameMode::Easy);
                                        logger.info("updated setting 'game mode': Easy");
                                    }
                                >
                                    "Easy"
                                </button>
                                <button
                                    class:active=move || game_mode.get() == GameMode::Realism
                                    on:click=move |_| {
                                        game_mode.set(GameMode::Realism);
                                        logger.info("updated setting 'game mode': Realism");
                                    }
                                >
                                    "Realism"
                                </button>
                            </div>
                        </div>
                        <div class="mode-toggle">
                            <span>"Airport Code"</span>
                            <div class="toggle-options">
                                <button
                                    class:active=move || {
                                        settings.get().airport_code_pref == AirportCodePref::Iata
                                    }
                                    on:click=move |_| {
                                        settings
                                            .update(|s| s.airport_code_pref = AirportCodePref::Iata);
                                        logger.info("updated setting 'airport code': IATA");
                                    }
                                >
                                    "IATA"
                                </button>
                                <button
                                    class:active=move || {
                                        settings.get().airport_code_pref == AirportCodePref::Icao
                                    }
                                    on:click=move |_| {
                                        settings
                                            .update(|s| s.airport_code_pref = AirportCodePref::Icao);
                                        logger.info("updated setting 'airport code': ICAO");
                                    }
                                >
                                    "ICAO"
                                </button>
                            </div>
                        </div>
                        <label class="checkbox-label">
                            "Default 4x Speed"
                            <input
                                type="checkbox"
                                prop:checked=move || settings.get().default_4x
                                on:change=move |ev| {
                                    let c = ev
                                        .target()
                                        .unwrap()
                                        .unchecked_into::<HtmlInputElement>()
                                        .checked();
                                    settings.update(|s| s.default_4x = c);
                                    logger
                                        .info(format!("updated setting 'default 4x speed': {}", c));
                                }
                            />
                        </label>
                        <label class="checkbox-label">
                            "Default Speed Mod"
                            <input
                                type="checkbox"
                                prop:checked=move || settings.get().default_speed_mod
                                on:change=move |ev| {
                                    let c = ev
                                        .target()
                                        .unwrap()
                                        .unchecked_into::<HtmlInputElement>()
                                        .checked();
                                    settings.update(|s| s.default_speed_mod = c);
                                    logger
                                        .info(
                                            format!("updated setting 'default speed mod': {}", c),
                                        );
                                }
                            />
                        </label>
                        <label class="checkbox-label">
                            "Default Fuel Mod"
                            <input
                                type="checkbox"
                                prop:checked=move || settings.get().default_fuel_mod
                                on:change=move |ev| {
                                    let c = ev
                                        .target()
                                        .unwrap()
                                        .unchecked_into::<HtmlInputElement>()
                                        .checked();
                                    settings.update(|s| s.default_fuel_mod = c);
                                    logger
                                        .info(format!("updated setting 'default fuel mod': {}", c));
                                }
                            />
                        </label>
                        <label class="checkbox-label">
                            "Default CO2 Mod"
                            <input
                                type="checkbox"
                                prop:checked=move || settings.get().default_co2_mod
                                on:change=move |ev| {
                                    let c = ev
                                        .target()
                                        .unwrap()
                                        .unchecked_into::<HtmlInputElement>()
                                        .checked();
                                    settings.update(|s| s.default_co2_mod = c);
                                    logger
                                        .info(format!("updated setting 'default co2 mod': {}", c));
                                }
                            />
                        </label>
                        <label class="checkbox-label">
                            "Include flights exceeding 24hr"
                            <input
                                type="checkbox"
                                prop:checked=move || settings.get().allow_invalid_tpd
                                on:change=move |ev| {
                                    let c = ev
                                        .target()
                                        .unwrap()
                                        .unchecked_into::<HtmlInputElement>()
                                        .checked();
                                    settings.update(|s| s.allow_invalid_tpd = c);
                                    logger
                                        .info(
                                            format!("updated setting 'allow invalid tpd': {}", c),
                                        );
                                }
                            />
                        </label>
                    </div>

                    <div class="setting-group">
                        <h4>"Prices"</h4>
                        <ValidatedInput
                            label="Fuel Price"
                            value=move || u16::from(settings.get().fuel_price).to_string()
                            update=|s, v| {
                                v.parse::<u16>()
                                    .map(|x| s.fuel_price = FuelPrice::from(x))
                                    .map_err(|_| "Invalid integer".into())
                            }
                        />
                        <ValidatedInput
                            label="CO2 Price"
                            value=move || u16::from(settings.get().co2_price).to_string()
                            update=|s, v| {
                                v.parse::<u16>()
                                    .map(|x| s.co2_price = Co2Price::from(x))
                                    .map_err(|_| "Invalid integer".into())
                            }
                        />
                    </div>

                    <div class="setting-group">
                        <h4>"Training"</h4>
                        <ValidatedInput
                            label="Fuel (0-3)"
                            min="0"
                            max="3"
                            value=move || u8::from(settings.get().training.fuel).to_string()
                            update=|s, v| {
                                v.parse::<u8>()
                                    .map_err(|_| ValidationError::InvalidFuelTraining)
                                    .and_then(FuelTraining::new)
                                    .map(|x| s.training.fuel = x)
                                    .map_err(|e| e.to_string())
                            }
                        />
                        <ValidatedInput
                            label="CO2 (0-5)"
                            min="0"
                            max="5"
                            value=move || u8::from(settings.get().training.co2).to_string()
                            update=|s, v| {
                                v.parse::<u8>()
                                    .map_err(|_| ValidationError::InvalidCo2Training)
                                    .and_then(Co2Training::new)
                                    .map(|x| s.training.co2 = x)
                                    .map_err(|e| e.to_string())
                            }
                        />
                        <ValidatedInput
                            label="Repair (0-5)"
                            min="0"
                            max="5"
                            value=move || u8::from(settings.get().training.repair).to_string()
                            update=|s, v| {
                                v.parse::<u8>()
                                    .map_err(|_| ValidationError::InvalidRepairTraining)
                                    .and_then(RepairTraining::new)
                                    .map(|x| s.training.repair = x)
                                    .map_err(|e| e.to_string())
                            }
                        />
                        <ValidatedInput
                            label="Wear (0-5)"
                            min="0"
                            max="5"
                            value=move || u8::from(settings.get().training.wear).to_string()
                            update=|s, v| {
                                v.parse::<u8>()
                                    .map_err(|_| ValidationError::InvalidWearTraining)
                                    .and_then(WearTraining::new)
                                    .map(|x| s.training.wear = x)
                                    .map_err(|e| e.to_string())
                            }
                        />
                        <ValidatedInput
                            label="Large (0-6)"
                            min="0"
                            max="6"
                            value=move || u8::from(settings.get().training.l).to_string()
                            update=|s, v| {
                                v.parse::<u8>()
                                    .map_err(|_| ValidationError::InvalidLargeTraining)
                                    .and_then(LargeTraining::new)
                                    .map(|x| s.training.l = x)
                                    .map_err(|e| e.to_string())
                            }
                        />
                        <ValidatedInput
                            label="Heavy (0-6)"
                            min="0"
                            max="6"
                            value=move || u8::from(settings.get().training.h).to_string()
                            update=|s, v| {
                                v.parse::<u8>()
                                    .map_err(|_| ValidationError::InvalidHeavyTraining)
                                    .and_then(HeavyTraining::new)
                                    .map(|x| s.training.h = x)
                                    .map_err(|e| e.to_string())
                            }
                        />
                    </div>

                    <div class="setting-group">
                        <h4>"Configuration"</h4>
                        <ValidatedInput
                            label="Pax Load"
                            step="0.01"
                            value=move || format!("{:.3}", settings.get().load.get())
                            update=|s, v| {
                                v.parse::<f32>()
                                    .map_err(|_| ValidationError::InvalidAircraftLoad)
                                    .and_then(AircraftLoad::new)
                                    .map(|x| s.load = x)
                                    .map_err(|e| e.to_string())
                            }
                        />
                        <ValidatedInput
                            label="Cargo Load"
                            step="0.01"
                            value=move || format!("{:.3}", settings.get().cargo_load.get())
                            update=|s, v| {
                                v.parse::<f32>()
                                    .map_err(|_| ValidationError::InvalidAircraftLoad)
                                    .and_then(AircraftLoad::new)
                                    .map(|x| s.cargo_load = x)
                                    .map_err(|e| e.to_string())
                            }
                        />
                        <ValidatedInput
                            label="Loss Tolerance"
                            step="0.001"
                            value=move || format!("{:.3}", settings.get().revenue_loss_tol.get())
                            update=|s, v| {
                                v.parse::<f32>()
                                    .map_err(|_| ValidationError::InvalidRevenueLossTol)
                                    .and_then(RevenueLossTol::new)
                                    .map(|x| s.revenue_loss_tol = x)
                                    .map_err(|e| e.to_string())
                            }
                        />
                    </div>
                </div>
            </details>
        </div>
    }
}
