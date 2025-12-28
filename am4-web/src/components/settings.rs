use crate::console::UserLogger;
use am4::user::{
    AircraftLoad, Co2Price, Co2Training, FuelPrice, FuelTraining, GameMode, HeavyTraining,
    IncomeLossTol, LargeTraining, RepairTraining, Settings, ValidationError, WearTraining,
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
    let (is_invalid, set_invalid) = signal(false);

    view! {
        <label>
            {label}
            <input
                type="number"
                min=min
                max=max
                step=step
                class:invalid=is_invalid
                prop:value=value
                on:change=move |ev| {
                    let target = ev.target().unwrap();
                    let val = target.unchecked_into::<HtmlInputElement>().value();
                    settings
                        .update(|s| {
                            match update(s, val.clone()) {
                                Ok(_) => {
                                    set_invalid.set(false);
                                    logger
                                        .info(
                                            format!(
                                                "updated setting '{}': {}",
                                                label.to_lowercase(),
                                                val,
                                            ),
                                        );
                                }
                                Err(_) => set_invalid.set(true),
                            }
                        });
                }
            />
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
                        <label class="checkbox-label">
                            "4x Speed"
                            <input
                                type="checkbox"
                                prop:checked=move || settings.get().fourx
                                on:change=move |ev| {
                                    let c = ev
                                        .target()
                                        .unwrap()
                                        .unchecked_into::<HtmlInputElement>()
                                        .checked();
                                    settings.update(|s| s.fourx = c);
                                    logger.info(format!("updated setting '4x speed': {}", c));
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
                                    .map_err(|_| "err".into())
                            }
                        />
                        <ValidatedInput
                            label="CO2 Price"
                            value=move || u16::from(settings.get().co2_price).to_string()
                            update=|s, v| {
                                v.parse::<u16>()
                                    .map(|x| s.co2_price = Co2Price::from(x))
                                    .map_err(|_| "err".into())
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
                                    .map_err(|_| "err".into())
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
                                    .map_err(|_| "err".into())
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
                                    .map_err(|_| "err".into())
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
                                    .map_err(|_| "err".into())
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
                                    .map_err(|_| "err".into())
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
                                    .map_err(|_| "err".into())
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
                                    .map_err(|_| "err".into())
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
                                    .map_err(|_| "err".into())
                            }
                        />
                        <ValidatedInput
                            label="Loss Tolerance"
                            step="0.001"
                            value=move || format!("{:.3}", settings.get().income_loss_tol.get())
                            update=|s, v| {
                                v.parse::<f32>()
                                    .map_err(|_| ValidationError::InvalidIncomeLossTol)
                                    .and_then(IncomeLossTol::new)
                                    .map(|x| s.income_loss_tol = x)
                                    .map_err(|_| "err".into())
                            }
                        />
                    </div>
                </div>
            </details>
        </div>
    }
}
