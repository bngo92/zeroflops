use std::{collections::HashMap, rc::Rc};

use crate::{
    ListMode, ListsRoute, Route, UserProps,
    bootstrap::{Accordion, Collapse},
    dataframe::DataFrame,
};
use mybops::List;
use wasm_bindgen::JsValue;
use web_sys::HtmlSelectElement;
use yew::{Callback, Component, Context, Html, NodeRef, Properties, html};
use yew_router::{prelude::Link, scope_ext::RouterScopeExt};

pub enum HomeMsg {
    ToggleHelp,
    Load(Vec<List>),
    Create,
}

pub struct Home {
    help_collapsed: bool,
    lists: Vec<List>,
    select_ref: NodeRef,
}

impl Component for Home {
    type Message = HomeMsg;
    type Properties = UserProps;

    fn create(ctx: &Context<Self>) -> Self {
        let select_ref = NodeRef::default();
        ctx.link().send_future(Home::fetch_lists());
        Home {
            help_collapsed: ctx.props().logged_in,
            lists: Vec::new(),
            select_ref,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HomeMsg::ToggleHelp => {
                self.help_collapsed = !self.help_collapsed;
                true
            }
            HomeMsg::Load(lists) => {
                self.lists = lists;
                true
            }
            HomeMsg::Create => {
                let navigator = ctx.link().navigator().unwrap();
                ctx.link().send_future_batch(async move {
                    let list = crate::create_list(None).await.unwrap();
                    navigator.push(&ListsRoute::Edit { id: list.id });
                    None
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let disabled = !ctx.props().logged_in;
        let create = ctx.link().callback(|_| HomeMsg::Create);
        let mut column = Vec::new();
        let mut left = Vec::new();
        let mut right = Vec::new();
        for (l, &r) in self.lists.iter().zip([false, true].iter().cycle()) {
            column.push(html! {<Widget list={l.clone()} select_ref={self.select_ref.clone()}/>});
            if r {
                right.push(html! {<Widget list={l.clone()} select_ref={self.select_ref.clone()}/>});
            } else {
                left.push(html! {<Widget list={l.clone()} select_ref={self.select_ref.clone()}/>});
            }
        }
        crate::nav_content(
            html! {
              <>
                <ul class="navbar-nav me-auto">
                  <li class="navbar-brand">if disabled { {"Demo"} } else { { "Home" } }</li>
                </ul>
                <div class="d-flex gap-3 align-items-baseline">
                  <span class="navbar-text text-nowrap">{"Sort Mode:"}</span>
                  <select ref={self.select_ref.clone()} class="form-select">
                    <option>{"Tournament"}</option>
                    <option selected=true>{"Random Tournament"}</option>
                    <option>{"Random Matches"}</option>
                    <option>{"Random Rounds"}</option>
                  </select>
                  <button class="btn btn-info" onclick={ctx.link().callback(|_| HomeMsg::ToggleHelp)}>{"Help"}</button>
                </div>
              </>
            },
            html! {
              <div>
                <Collapse collapsed={self.help_collapsed}>
                  <p>
                  {"mybops is an app that helps you filter your data and remove flops from your life.
                    Use it to gain insights about your favorite songs, TV shows, and even restaurants.
                    mybops makes it easy to rate and/or rank what's important to you."}
                  </p>
                  <p>
                  {"The data is organized into lists of items and your lists are displayed here on the home page using user-defined widgets.
                    The fastest way to rank your items is with a randomly generated tournament.
                    You can start a tournament for a list by clicking the "}<button type="button" class="btn btn-success btn-sm">{"Rank"}</button>
                    {" button below the list widget. Here is the full list of sort modes:"}
                  </p>
                  <ul>
                    <li><strong>{"Tournament"}</strong>{" - Sort by choosing between items that are organized using a seeded tournament."}</li>
                    <li><strong>{"Random Tournament"}</strong>{" - Sort by choosing between items that are organized using a randomly generated tournament."}</li>
                    <li><strong>{"Random Matches"}</strong>{" - Sort by choosing between randomly selected items."}</li>
                    <li><strong>{"Random Rounds"}</strong>{" - This mode is similar to Random Matches except every item will be selected before an item is repeated."}</li>
                  </ul>
                  <p>{"To rate items, go to the item rating page for the list by clicking on the "}<button type="button" class="btn btn-success btn-sm">{"Rate"}</button>{" button."}</p>
                  <p>{"You can also:"}</p>
                  <ul class="mb-0">
                    <li>{"View items in the list by clicking on the widget to expand it."}</li>
                    <li>{"Search for data about your ratings and rankings by going to the "}<Link<Route> to={Route::Search}>{"Search"}</Link<Route>>{" page."}</li>
                  </ul>
                </Collapse>
                <div class="mt-3">
                  <div class="d-md-none">
                    {column}
                    <button type="button" class="btn btn-primary" onclick={create.clone()} {disabled}>{"Create List"}</button>
                  </div>
                  <div class="d-none d-md-block">
                    <div class="d-grid gap-3" style="grid-template-columns: 1fr 1fr">
                      <div>
                        {left}
                        <button type="button" class="btn btn-primary" onclick={create} {disabled}>{"Create List"}</button>
                      </div>
                      <div>{right}</div>
                    </div>
                  </div>
                </div>
              </div>
            },
        )
    }
}

impl Home {
    async fn fetch_lists() -> HomeMsg {
        let lists = crate::fetch_lists(true).await.unwrap();
        HomeMsg::Load(lists)
    }
}

enum WidgetMsg {
    Fetching(Rc<List>),
    Success(Option<DataFrame>),
}

#[derive(PartialEq, Properties)]
pub struct WidgetProps {
    list: List,
    select_ref: NodeRef,
}

struct Widget {
    collapsed: bool,
    query: Option<DataFrame>,
}

impl Component for Widget {
    type Message = WidgetMsg;
    type Properties = WidgetProps;

    fn create(_: &Context<Self>) -> Self {
        Widget {
            collapsed: true,
            query: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            WidgetMsg::Fetching(list) => {
                // TODO: add the ability to refresh
                if self.query.is_none() {
                    ctx.link().send_future(async move {
                        WidgetMsg::Success(crate::query_list(&list, None).await.unwrap())
                    });
                    false
                } else {
                    self.collapsed = !self.collapsed;
                    true
                }
            }
            WidgetMsg::Success(query) => {
                self.collapsed = false;
                self.query = query;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let list = &ctx.props().list;
        let on_toggle = ctx.link().callback({
            let list = Rc::new(list.clone());
            move |_| WidgetMsg::Fetching(Rc::clone(&list))
        });
        let navigator = ctx.link().navigator().unwrap();
        let select_ref = ctx.props().select_ref.clone();
        let navigator_copy = navigator.clone();
        let id = list.id.clone();
        let compare = Callback::from(move |_| {
            let id = id.clone();
            let mode = select_ref.cast::<HtmlSelectElement>().unwrap().value();
            match mode.as_ref() {
                "Random Matches" => {
                    navigator_copy.push(&ListsRoute::Match { id });
                }
                "Random Rounds" => {
                    navigator_copy
                        .push_with_query(
                            &ListsRoute::Match { id },
                            &[("mode", "rounds")].into_iter().collect::<HashMap<_, _>>(),
                        )
                        .unwrap();
                }
                "Tournament" => {
                    navigator_copy.push(&ListsRoute::Tournament { id });
                }
                "Random Tournament" => {
                    navigator_copy
                        .push_with_query(
                            &ListsRoute::Tournament { id },
                            &[("mode", "random")].into_iter().collect::<HashMap<_, _>>(),
                        )
                        .unwrap();
                }
                _ => {
                    web_sys::console::log_1(&JsValue::from("Invalid mode"));
                }
            };
        });
        let id = list.id.clone();
        let go = Callback::from(move |_| {
            navigator.push(&ListsRoute::List { id: id.clone() });
        });
        // TODO: support actions on views
        let disabled = matches!(list.mode, ListMode::View(_));
        html! {
            <>
                <Accordion header={list.name.clone()} collapsed={self.collapsed} {on_toggle}>
                    if let Some(query) = &self.query {
                        {crate::plot::df_table_view(query, false)}
                    } else {
                        <div></div>
                    }
                </Accordion>
                <div class="row mb-3">
                    <div class="col-auto">
                        <button type="button" class="btn btn-success" onclick={go} {disabled}>{"Rate"}</button>
                    </div>
                    <div class="col-auto">
                        <button type="button" class="btn btn-success" onclick={compare} {disabled}>{"Rank"}</button>
                    </div>
                </div>
            </>
        }
    }
}
