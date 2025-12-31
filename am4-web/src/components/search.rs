use leptos::html::Input;
use leptos::prelude::*;
use std::time::Duration;
use web_sys::KeyboardEvent;

#[component]
pub fn MultiSelect<T, ViewOpt, ViewPill>(
    #[prop(into)] selected: RwSignal<Vec<T>>,
    #[prop(into)] active: RwSignal<Option<T>>,
    #[prop(into)] search: Callback<String, Vec<T>>,
    render_option: ViewOpt,
    render_pill: ViewPill,
    #[prop(optional, into)] max_items: Option<usize>,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional, into)] is_selectable: Option<Callback<T, bool>>,
) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
    ViewOpt: Fn(
            T,
            usize,
            Memo<Vec<T>>,
            ReadSignal<usize>,
            Callback<T>,
            Callback<T>,
            Callback<Box<dyn Fn(T) -> T + Send + Sync>>,
        ) -> AnyView
        + 'static
        + Copy
        + Send
        + Sync,
    ViewPill: Fn(T, Callback<()>) -> AnyView + 'static + Copy + Send + Sync,
{
    let (query, set_query) = signal(String::new());
    let (is_focused, set_focused) = signal(false);
    let (highlight_idx, set_highlight_idx) = signal(0);
    let (pinned, set_pinned) = signal(None::<T>);
    let input_ref = NodeRef::<Input>::new();
    let suggestions_override = RwSignal::new(None::<Vec<T>>);

    let base_suggestions = Memo::new(move |_| {
        let q = query.get();
        if q.is_empty() {
            if pinned.get().is_none() {
                active.set(None);
            }
            suggestions_override.set(None);
            vec![]
        } else {
            suggestions_override.set(None);
            search.run(q)
        }
    });

    let suggestions = Memo::new(move |_| {
        suggestions_override
            .get()
            .unwrap_or_else(|| base_suggestions.get())
    });

    Effect::new(move |_| {
        suggestions.track();
        if let Some(cb) = is_selectable {
            let suggs = suggestions.get_untracked();
            if let Some(idx) = suggs.iter().position(|item| cb.run(item.clone())) {
                set_highlight_idx.set(idx);
            } else {
                set_highlight_idx.set(0);
            }
        } else {
            set_highlight_idx.set(0);
        }
    });

    let select = move |item: T| {
        if let Some(cb) = is_selectable {
            if !cb.run(item.clone()) {
                return;
            }
        }

        selected.update(|s| {
            if let Some(max) = max_items {
                if s.len() >= max {
                    s.pop();
                }
            }
            if !s.contains(&item) {
                s.push(item.clone());
            }
        });
        set_query.set(String::new());
        set_pinned.set(None);
        active.set(None);
        suggestions_override.set(None);
        if let Some(input) = input_ref.get() {
            input.focus().ok();
        }
    };

    let remove = move |item: T| {
        selected.update(|s| {
            if let Some(pos) = s.iter().position(|x| *x == item) {
                s.remove(pos);
            }
        });
        if pinned.get() == Some(item) {
            set_pinned.set(None);
            active.set(None);
        }
    };

    let on_keydown = move |ev: KeyboardEvent| {
        let suggs = suggestions.get();
        if suggs.is_empty() {
            if ev.key() == "Backspace" && query.get().is_empty() {
                selected.update(|s| {
                    if let Some(item) = s.pop() {
                        if pinned.get() == Some(item) {
                            set_pinned.set(None);
                            active.set(None);
                        }
                    }
                });
            }
            return;
        }

        match ev.key().as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                set_highlight_idx.update(|i| {
                    let mut next = *i + 1;
                    while next < suggs.len() {
                        let ok = is_selectable.is_none_or(|cb| cb.run(suggs[next].clone()));
                        if ok {
                            *i = next;
                            return;
                        }
                        next += 1;
                    }
                });
            }
            "ArrowUp" => {
                ev.prevent_default();
                set_highlight_idx.update(|i| {
                    if *i > 0 {
                        let mut prev = *i - 1;
                        loop {
                            let ok = is_selectable.is_none_or(|cb| cb.run(suggs[prev].clone()));
                            if ok {
                                *i = prev;
                                return;
                            }
                            if prev == 0 {
                                break;
                            }
                            prev -= 1;
                        }
                    }
                });
            }
            "Enter" => {
                ev.prevent_default();
                if let Some(item) = suggs.get(highlight_idx.get()) {
                    select(item.clone());
                }
            }
            _ => {}
        }
    };

    let on_pill_click = move |item: T| {
        if pinned.with(|p| p.as_ref() == Some(&item)) {
            set_pinned.set(None);
        } else {
            set_pinned.set(Some(item.clone()));
            active.set(Some(item));
        }
    };

    view! {
        <div
            class="search-container"
            on:click=move |_| {
                if let Some(input) = input_ref.get() {
                    input.focus().ok();
                }
            }
        >
            <div class="pills">
                <For
                    each=move || selected.get()
                    key=|_| uuid::Uuid::new_v4()
                    children=move |item| {
                        let item_for_active = item.clone();
                        let item_for_enter = item.clone();
                        let item_for_click = item.clone();
                        let render_pill = render_pill;
                        view! {
                            <div
                                class="pill"
                                class:active=move || {
                                    active.with(|a| a.as_ref() == Some(&item_for_active))
                                }
                                on:mouseenter=move |_| active.set(Some(item_for_enter.clone()))
                                on:mouseleave=move |_| active.set(pinned.get())
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    on_pill_click(item_for_click.clone());
                                }
                            >
                                {render_pill(
                                    item.clone(),
                                    Callback::new(move |_| remove(item.clone())),
                                )}
                            </div>
                        }
                    }
                />
                <div class="input-wrapper">
                    <input
                        node_ref=input_ref
                        type="text"
                        placeholder=move || {
                            if selected.get().is_empty() {
                                placeholder.clone()
                            } else {
                                String::new()
                            }
                        }
                        prop:value=query
                        on:input=move |ev| set_query.set(event_target_value(&ev))
                        on:focus=move |_| set_focused.set(true)
                        on:blur=move |_| {
                            set_timeout(move || set_focused.set(false), Duration::from_millis(150));
                        }
                        on:keydown=on_keydown
                    />
                </div>
            </div>

            <Show when=move || is_focused.get() && !suggestions.with(|s| s.is_empty())>
                <div class="dropdown">
                    <For
                        each=move || suggestions.get().into_iter().enumerate()
                        key=|_| uuid::Uuid::new_v4()
                        children=move |(i, item)| {
                            let item_for_click = item.clone();
                            let item_for_update = item.clone();
                            let render_option = render_option;
                            let select = select;
                            let update_item = move |new_item: T| {
                                suggestions_override
                                    .update(|ov| {
                                        let mut current = ov
                                            .take()
                                            .unwrap_or_else(|| base_suggestions.get());
                                        if let Some(pos) = current
                                            .iter()
                                            .position(|x| *x == item_for_update)
                                        {
                                            current[pos] = new_item;
                                        }
                                        *ov = Some(current);
                                    });
                            };
                            let update_all = move |transform: Box<dyn Fn(T) -> T + Send + Sync>| {
                                suggestions_override
                                    .update(|ov| {
                                        let current = ov
                                            .take()
                                            .unwrap_or_else(|| base_suggestions.get());
                                        let updated: Vec<T> = current
                                            .into_iter()
                                            .map(&transform)
                                            .collect();
                                        *ov = Some(updated);
                                    });
                            };

                            view! {
                                <div
                                    class="option"
                                    class:selected=move || i == highlight_idx.get()
                                    on:click=move |_| select(item_for_click.clone())
                                    on:mouseenter=move |_| set_highlight_idx.set(i)
                                    on:mousedown=move |ev| ev.prevent_default()
                                >
                                    {render_option(
                                        item,
                                        i,
                                        suggestions,
                                        highlight_idx,
                                        Callback::new(select),
                                        Callback::new(update_item),
                                        Callback::new(update_all),
                                    )}
                                </div>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}
