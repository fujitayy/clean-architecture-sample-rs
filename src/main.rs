//! layered
//!
//! ## これは何？
//!
//! Cake Pattern + Clean Architecture っぽいやつのサンプルコード
//!
//! ## Rustの機能を一部解説
//!
//! * `let 変数名`: 変数宣言。変数はデフォルトでimmutable.
//! * `let mut 変数名`: 変数宣言。mutableな変数を宣言する時は `mut` を付ける。
//! * `&変数名`: immutable参照。mutable参照は `&mut 変数名`. immutable参照が存在している間はmutable参照を作れない（同時に存在できるmutableな変数は1個まで）。
//! * `hoge() -> 戻り値型`: `->` は戻り値の型はこれですよの意味。戻り値を返さない場合は `-> 戻り値型` を省略可能。戻り値型を省略して `->` だけにすると戻ってこない関数を表せる。
//! * `Result<何か, Error>`: Goで関数の戻り値に書く `何か, error` がもっと便利になったやつ。どう便利なのかはここでは扱わない。
//! * `pub`: pubと書くとモジュールの外から参照できるようになる。structのフィールドにも個別に付ける必要あり。
//! * `mod`: モジュール定義。C++のnamespaceと似てるがもう少し便利になってる。今回は1ファイルの中に全て書いてますが、modの中は別ファイルに分けることが出来ます。
//! * `trait`: デフォルト実装を定義できるinterface。ある型にあるtraitを実装(impl)するのは型定義の外で行える為、第三者が定義した型に自分が定義したtraitを実装(impl)する事が可能。
//! * `impl trait名 for 型名`: traitを`型名`用に実装している。
//! * `?`: 関数・メソッドの末尾にたまに付いてる。Goで `if err != nil { return err; }` と書いてるアレのシンタックスシュガー。実際にはもう少し複雑な処理を行っている(戻り値のError部分の型へ自動で型変換するとか)。
//! * `|引数| 式`: ラムダ式（無名関数）
//! * `#[derive(trait名, trait名, ...)]`: 実装(impl)をコンパイラが自動導出可能な一部のtraitは、こんな風な呪文を型定義の頭に付ける事でコンパイラが自動で実装してくれる。Goでいうstringer。
//! * `use`: モジュール内のアイテムの読み込み。
//! * Rustは関数の最後の式にセミコロンを付けない場合、その式の戻り値を関数の戻り値として返します。

extern crate chrono;
extern crate failure;

mod component {
    //! ストレージアクセス、DBアクセス、現在時刻取得、ネットワークアクセス等の(多くの場合IOを伴う副作用を持つ)処理をcomponentとしてまとめる。
    //! Clean Architecture の円形の図で言うと最も外側に当たるレイヤ。

    pub mod time {
        use chrono::prelude::*;

        /// 現在時間取得処理を行うレイヤ
        pub trait TimeComponent {
            fn now(&self) -> DateTime<Local>;
        }

        /// これを実装(impl)している型はTimeComponentを返せる。抽象化されたGetter.
        pub trait HaveTimeComponent {
            type TimeComponent: TimeComponent;
            fn time_component(&self) -> &Self::TimeComponent;
        }

        /// TimeComponentをchronoを使って実装(impl)する型
        pub struct Chrono;

        impl TimeComponent for Chrono {
            fn now(&self) -> DateTime<Local> {
                Local::now()
            }
        }
    }

    pub mod storage {
        use entity::user::{Name, User};
        use failure::Error;
        use std::collections::BTreeMap;

        /// ユーザー情報をストレージに出し入れするレイヤ
        pub trait UserStorageComponent {
            fn read(&self, Name) -> Result<User, Error>;
            fn save(&mut self, Name, User) -> Result<(), Error>;
            fn read_all(&self) -> Result<Vec<User>, Error>;
            fn save_all(&mut self, users: &[(Name, User)]) -> Result<(), Error>;
        }

        /// これを実装(impl)している型はUserStorageComponentを返せる。抽象化されたGetter.
        /// 引数の型が&selfの方は参照only。mutはmutableの略で、これが付いてると値の変更が可能。
        pub trait HaveUserStorageComponent {
            type UserStorageComponent: UserStorageComponent;
            fn user_storage_component(&self) -> &Self::UserStorageComponent;
            fn user_storage_component_mut(&mut self) -> &mut Self::UserStorageComponent;
        }

        /// メモリ上に値を保持するストレージ抽象型
        pub struct MemoryStorage {
            list: BTreeMap<Name, User>,
        }

        /// MemoryStorage型のメソッドを定義
        impl MemoryStorage {
            pub fn new() -> MemoryStorage {
                MemoryStorage {
                    list: BTreeMap::new(),
                }
            }
        }

        /// MemoryStorage型用のUserStorageComponentの実装(impl)
        impl UserStorageComponent for MemoryStorage {
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
        //! Cacheとかしたい場合はCacheComponentとHaveCacheComponentを定義して、UserRepositoryの制約に加える。
        //! 実際のプロダクトではこの辺のレイヤはもっと泥臭い感じになると思う

        use component::storage::{UserStorageComponent, HaveUserStorageComponent};
        use component::time::{TimeComponent, HaveTimeComponent};
        use entity::user::{Email, Name, User};
        use failure::Error;

        /// `HaveUserStorageComponent + HaveTimeComponent` は、+の左右のtraitを実装(impl)している型だけが、
        /// UserRepositoryを実装できる事を意味している。
        pub trait UserRepository: HaveUserStorageComponent + HaveTimeComponent {
            fn get(&self, name: Name) -> Result<User, Error> {
                Ok(self.user_storage_component().read(name)?)
            }

            fn insert(&mut self, name: Name, email: Email) -> Result<(), Error> {
                let now = self.time_component().now();
                let user = User {
                    name: name.clone(),
                    email,
                    create_time: now,
                    update_time: now,
                };
                self.user_storage_component_mut().save(name, user)?;
                Ok(())
            }
        }

        pub trait HaveUserRepository {
            type UserRepository: UserRepository;
            fn user_repository(&self) -> &Self::UserRepository;
            fn user_repository_mut(&mut self) -> &mut Self::UserRepository;
        }

        /// traitの実装(impl)は具象型だけでなくジェネリクスのパラメータのみで実装する事も出来る。
        /// これにより特定の条件を満たしている型全ての実装(impl)を用意する事が簡単に行える。
        impl<T: HaveUserStorageComponent + HaveTimeComponent> UserRepository for T {}
    }
}

mod entity {
    //! 一意性を持つデータを抽象化するレイヤ。
    //! 一意性を持たない場合は値として扱い、entityにはしない（数値の1とか文字列とかと同じ扱いにする）

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

mod env {
    use component::time::{HaveTimeComponent, Chrono};
    use component::storage::{HaveUserStorageComponent, MemoryStorage};
    use repository::users::{HaveUserRepository};

    /// Cake Pattern での環境型
    /// この構造体に各レイヤーを担当するオブジェクトを格納する。
    pub struct RealWorld {
        time_component: Chrono,
        storage_component: MemoryStorage,
    }

    impl RealWorld {
        pub fn new() -> RealWorld {
            RealWorld {
                time_component: Chrono,
                storage_component: MemoryStorage::new(),
            }
        }
    }

    impl HaveTimeComponent for RealWorld {
        type TimeComponent = Chrono;
        fn time_component(&self) -> &Chrono {
            &self.time_component
        }
    }

    impl HaveUserStorageComponent for RealWorld {
        type UserStorageComponent = MemoryStorage;
        fn user_storage_component(&self) -> &MemoryStorage {
            &self.storage_component
        }

        fn user_storage_component_mut(&mut self) -> &mut MemoryStorage {
            &mut self.storage_component
        }
    }

    impl HaveUserRepository for RealWorld {
        type UserRepository = Self;
        fn user_repository(&self) -> &Self {
            self
        }

        fn user_repository_mut(&mut self) -> &mut Self {
            self
        }
    }
}

fn main() {
    use repository::users::{UserRepository, HaveUserRepository};
    use entity::user::{Email, Name};
    use env::RealWorld;

    let mut app = RealWorld::new();

    let name = Name {
        name: "user_a".to_string(),
    };

    app.user_repository_mut().insert(
        name.clone(),
        Email {
            email: "user_a@example.com".to_string(),
        },
    ).unwrap();
    println!("{:?}", app.get(name));
}

#[cfg(test)]
mod tests {
    mod mock {
        pub mod time {
            use chrono::prelude::*;
            use component::time::TimeComponent;
            use std::str::FromStr;

            /// テスト用のTimeComponent実装。
            /// now()が特定の日時しか返さない。
            pub struct MockTime;

            impl TimeComponent for MockTime {
                fn now(&self) -> DateTime<Local> {
                    DateTime::from_str("2018-08-20T10:00:00 +0900").unwrap()
                }
            }
        }

        pub mod env {
            use super::time::MockTime;
            use component::time::HaveTimeComponent;
            use component::storage::{HaveUserStorageComponent, MemoryStorage};
            use repository::users::{HaveUserRepository};

            /// テスト用の Cake Pattern での環境型
            /// この構造体に各レイヤーを担当するオブジェクトを格納する。
            pub struct TestWorld {
                time_component: MockTime,
                storage_component: MemoryStorage,
            }

            impl TestWorld {
                pub fn new() -> TestWorld {
                    TestWorld {
                        time_component: MockTime,
                        storage_component: MemoryStorage::new(),
                    }
                }
            }

            impl HaveTimeComponent for TestWorld {
                type TimeComponent = MockTime;
                fn time_component(&self) -> &MockTime {
                    &self.time_component
                }
            }

            impl HaveUserStorageComponent for TestWorld {
                type UserStorageComponent = MemoryStorage;
                fn user_storage_component(&self) -> &MemoryStorage {
                    &self.storage_component
                }

                fn user_storage_component_mut(&mut self) -> &mut MemoryStorage {
                    &mut self.storage_component
                }
            }

            impl HaveUserRepository for TestWorld {
                type UserRepository = Self;
                fn user_repository(&self) -> &Self {
                    self
                }

                fn user_repository_mut(&mut self) -> &mut Self {
                    self
                }
            }
        }
    }

    use self::mock::env::TestWorld;
    use chrono::prelude::*;
    use entity::user::{Email, Name};
    use repository::users::{UserRepository, HaveUserRepository};
    use std::str::FromStr;

    #[test]
    fn add_user() {
        let mut app = TestWorld::new();

        let name = Name {
            name: "user1".to_string(),
        };
        let email = Email {
            email: "user1@example.com".to_string(),
        };

        app.user_repository_mut().insert(name.clone(), email.clone()).unwrap();

        let user = app.user_repository().get(name.clone()).unwrap();
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
