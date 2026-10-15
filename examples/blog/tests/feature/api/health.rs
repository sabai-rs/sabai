use sabai_testing::route;

use crate::app;

const ROUTE: &str = "health";

#[test]
fn health_answers_when_the_database_does() {
    let response = app().get_json(route(ROUTE));

    response.assert_status(200);
}
