//! layered
//!
//! レイヤー構造をジェネリクスとtraitで表現する
//! 示したいものは monad transformer の様な物

extern crate chrono;
#[macro_use]
extern crate failure;

/// 実装がLocalTImeなのかUTCなのかタイムゾーン無しなのかは一旦考えない

mod io {
    pub mod time {
        use chrono::prelude::*;

        pub trait TimeProvider {
            fn now(&self) -> DateTime<Local>;
        }

        /// presentationレイヤーとしてのchronoに相当する何かを型で定義する
        pub struct ChronoTimeProvider;

        impl TimeProvider for ChronoTimeProvider {
            fn now(&self) -> DateTime<Local> {
                Local::now()
            }
        }
    }

    pub mod storage {
        use entity::user::{Name, User};
        use failure::Error;
        use std::collections::BTreeMap;

        pub trait UserStorageProvider {
            fn read(&self, Name) -> Result<User, Error>;
            fn save(&mut self, Name, User) -> Result<(), Error>;
            fn read_all(&self) -> Result<Vec<User>, Error>;
            fn save_all(&mut self, users: &[(Name, User)]) -> Result<(), Error>;
        }

        pub struct MemoryUserStorage {
            list: BTreeMap<Name, User>,
        }

        impl MemoryUserStorage {
            pub fn new() -> MemoryUserStorage {
                MemoryUserStorage {
                    list: BTreeMap::new(),
                }
            }
        }

        impl UserStorageProvider for MemoryUserStorage {
            fn read(&self, name: Name) -> Result<User, Error> {
                Ok(self.list.get(&name).unwrap().clone())
            }

            fn save(&mut self, name: Name, user: User) -> Result<(), Error> {
                self.list.insert(name, user).map(|_| ());
                Ok(())
            }

            fn read_all(&self) -> Result<Vec<User>, Error> {
                Ok(self.list.values().map(|v| v.clone()).collect())
            }

            fn save_all(&mut self, users: &[(Name, User)]) -> Result<(), Error> {
                for (name, user) in users {
                    self.list.insert(name.clone(), user.clone());
                }
                Ok(())
            }
        }
    }
}

mod repository {
    pub mod users {
        use entity::user::{Email, Name, User};
        use failure::Error;
        use io::storage::UserStorageProvider;
        use io::time::TimeProvider;
        use std::collections::BTreeMap;

        /// ユーザー情報を管理する型
        pub struct Users<T, S> {
            list: BTreeMap<Name, User>,
            time_provider: T,
            storage_provider: S,
        }

        impl<T, S> Users<T, S>
        where
            T: TimeProvider,
            S: UserStorageProvider,
        {
            pub fn new(time_provider: T, storage_provider: S) -> Users<T, S> {
                Users {
                    list: BTreeMap::new(),
                    time_provider,
                    storage_provider,
                }
            }

            pub fn get_with_cache(&mut self, name: Name) -> Result<&User, Error> {
                let user = self.storage_provider.read(name)?;
                let name = user.name.clone();
                self.list.insert(name.clone(), user.clone());
                Ok(self.list.get(&name).unwrap())
            }

            pub fn insert(&mut self, name: Name, email: Email) -> Result<(), Error> {
                let user = User {
                    name: name.clone(),
                    email,
                    create_time: self.time_provider.now(),
                    update_time: self.time_provider.now(),
                };
                self.list.insert(name.clone(), user.clone());
                self.storage_provider.save(name, user)?;
                Ok(())
            }
        }
    }
}

mod entity {
    pub mod user {
        use chrono::prelude::*;

        /// アカウント1つを表す型
        #[derive(Debug, Clone)]
        pub struct User {
            pub name: Name,
            pub email: Email,
            pub create_time: DateTime<Local>,
            pub update_time: DateTime<Local>,
        }

        #[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
        pub struct Name {
            pub name: String,
        }

        #[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
        pub struct Email {
            pub email: String,
        }
    }
}

fn main() {
    use entity::user::{Email, Name};
    use repository::users::Users;

    let mut users = Users::new(
        io::time::ChronoTimeProvider,
        io::storage::MemoryUserStorage::new(),
    );
    let name = Name {
        name: "user_a".to_string(),
    };
    users.insert(
        name.clone(),
        Email {
            email: "user_a@example.com".to_string(),
        },
    ).unwrap();
    println!("{:?}", users.get_with_cache(name));
}

#[cfg(test)]
mod tests {
    mod mock {
        pub mod time {
            use chrono::prelude::*;
            use std::str::FromStr;

            pub struct MockTimeProvider;

            impl ::io::time::TimeProvider for MockTimeProvider {
                fn now(&self) -> DateTime<Local> {
                    DateTime::from_str("2018-08-20T10:00:00 +0900").unwrap()
                }
            }
        }
    }

    use super::*;
    use chrono::prelude::*;
    use entity::user::{Email, Name};
    use std::str::FromStr;

    #[test]
    fn add_user() {
        let name = Name {
            name: "user1".to_string(),
        };
        let email = Email {
            email: "user1@example.com".to_string(),
        };
        let mut users = repository::users::Users::new(
            mock::time::MockTimeProvider,
            io::storage::MemoryUserStorage::new(),
        );
        users.insert(name.clone(), email.clone()).unwrap();
        let user = users.get_with_cache(name.clone()).unwrap();
        assert_eq!(user.name, name);
        assert_eq!(user.email, email);
        assert_eq!(
            user.create_time,
            DateTime::<Local>::from_str("2018-08-20T10:00:00 +0900").unwrap()
        );
        assert_eq!(
            user.update_time,
            DateTime::<Local>::from_str("2018-08-20T10:00:00 +0900").unwrap()
        );
    }
}
