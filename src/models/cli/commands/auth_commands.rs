use super::{
    Database,
};



pub fn change_root_password(db: &mut Database) -> bool {
    db.change_master_password_independent();
    true
}