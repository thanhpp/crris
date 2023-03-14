use diesel::prelude::*;
use serde::Serialize;

use crate::schema::messages;

#[derive(Queryable, Insertable, Clone, Serialize)]
#[diesel(table_name = messages)]
pub struct Message {
    pub timestamp: i64,
    pub username: String,
    pub message: String,
}

pub fn create_message(conn: &mut PgConnection, msg: &Message) -> Result<(), diesel::result::Error> {
    match diesel::insert_into(messages::table)
        .values(msg)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_message() {
        pub fn establish_connection() -> Result<PgConnection, ConnectionError> {
            PgConnection::establish("postgres://username:password@localhost:5432/demo")
        }

        let conn = &mut establish_connection().expect("connect db error");
        create_message(
            conn,
            &Message {
                timestamp: 1,
                username: "thanhpp".into(),
                message: "hello, world".into(),
            },
        )
        .unwrap();
    }
}
