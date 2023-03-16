use diesel::{
    prelude::*,
    sql_query,
    sql_types::{BigInt, Text},
};
use serde::Serialize;

use crate::schema::messages;

#[derive(Queryable, Insertable, Clone, Serialize, QueryableByName, Debug)]
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

pub fn get_message_by_user(
    conn: &mut PgConnection,
    usrname: String,
    from: i64,
    to: i64,
) -> Result<Vec<Message>, diesel::result::Error> {
    let q = sql_query("SELECT * FROM messages WHERE username = $1 AND timestamp BETWEEN $2 AND $3");
    let msgs = match q
        .bind::<Text, _>(usrname)
        .bind::<BigInt, _>(from) // does not implement for i64, kinda weird
        .bind::<BigInt, _>(to) // does not implement for i64, kinda weird
        .get_results(conn)
    {
        Ok(x) => x,
        Err(e) => return Err(e),
    };

    Ok(msgs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn establish_connection() -> Result<PgConnection, ConnectionError> {
        PgConnection::establish("postgres://username:password@localhost:5432/demo")
    }
    #[test]
    fn test_create_message() {
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

    #[test]
    fn test_get_message_by_user() {
        let conn = &mut establish_connection().expect("connect db error");
        get_message_by_user(conn, "thanhpp".into(), 0, 1).unwrap();
    }
}
