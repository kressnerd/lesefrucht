//! `@e2e` step definitions: the real MCP surface over Streamable HTTP.

use cucumber::{given, then, when};
use serde_json::{Value, json};

use super::support::world::World;

fn passages(value: &Value) -> &Vec<Value> {
    value
        .get("passages")
        .and_then(|value| value.as_array())
        .expect("candidate passages")
}

fn citation(passage: &Value) -> &Value {
    passage.get("citation").expect("citation")
}

fn first_search(world: &World) -> &Value {
    world.e2e_search.as_ref().expect("search result")
}

fn first_context(world: &World) -> &Value {
    world.e2e_context.as_ref().expect("passage context")
}
