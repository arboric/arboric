//! GraphQL schema: a Star Wars `Character` interface with `Human` and `Droid`.

use juniper::{
    EmptySubscription, FieldResult, GraphQLEnum, RootNode, graphql_interface, graphql_object,
};

use crate::data::{Database, DroidData, HumanData};

#[derive(Debug, Clone, Copy, GraphQLEnum)]
pub enum Episode {
    NewHope,
    Empire,
    Jedi,
}

#[graphql_interface(for = [Human, Droid], context = Database)]
pub trait Character {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn friends(&self, ctx: &Database) -> Vec<CharacterValue>;
}

fn friends_of(ids: &[&str], ctx: &Database) -> Vec<CharacterValue> {
    ids.iter()
        .filter_map(|id| {
            ctx.human(id)
                .map(|h| Human(h.clone()).into())
                .or_else(|| ctx.droid(id).map(|d| Droid(d.clone()).into()))
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct Human(HumanData);

#[graphql_object(impl = CharacterValue, context = Database)]
impl Human {
    fn id(&self) -> &str {
        self.0.id
    }
    fn name(&self) -> &str {
        self.0.name
    }
    fn friends(&self, ctx: &Database) -> Vec<CharacterValue> {
        friends_of(self.0.friend_ids, ctx)
    }
    fn home_planet(&self) -> Option<&str> {
        self.0.home_planet
    }
}

impl Character for Human {
    fn id(&self) -> &str {
        self.0.id
    }
    fn name(&self) -> &str {
        self.0.name
    }
    fn friends(&self, ctx: &Database) -> Vec<CharacterValue> {
        friends_of(self.0.friend_ids, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct Droid(DroidData);

#[graphql_object(impl = CharacterValue, context = Database)]
impl Droid {
    fn id(&self) -> &str {
        self.0.id
    }
    fn name(&self) -> &str {
        self.0.name
    }
    fn friends(&self, ctx: &Database) -> Vec<CharacterValue> {
        friends_of(self.0.friend_ids, ctx)
    }
    fn primary_function(&self) -> Option<&str> {
        self.0.primary_function
    }
}

impl Character for Droid {
    fn id(&self) -> &str {
        self.0.id
    }
    fn name(&self) -> &str {
        self.0.name
    }
    fn friends(&self, ctx: &Database) -> Vec<CharacterValue> {
        friends_of(self.0.friend_ids, ctx)
    }
}

pub struct Query;

#[graphql_object(context = Database)]
impl Query {
    /// The hero of an episode. Like the dotnet example, always R2-D2.
    fn hero(ctx: &Database, episode: Option<Episode>) -> CharacterValue {
        let _ = episode;
        Droid(ctx.droid("3").expect("R2-D2 exists").clone()).into()
    }

    fn human(ctx: &Database, id: String) -> FieldResult<Option<Human>> {
        Ok(ctx.human(&id).map(|h| Human(h.clone())))
    }

    fn droid(ctx: &Database, id: String) -> FieldResult<Option<Droid>> {
        Ok(ctx.droid(&id).map(|d| Droid(d.clone())))
    }
}

pub struct Mutation;

#[graphql_object(context = Database)]
impl Mutation {
    /// Does nothing. Exists so that Arboric's mutation policies can be tested.
    fn ping() -> bool {
        true
    }
}

pub type Schema = RootNode<Query, Mutation, EmptySubscription<Database>>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::new())
}
