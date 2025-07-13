// event/model/macros.rs
// 最外层是文件，不是 mod 了

#[macro_export(local_inner_macros)] // 保证局部调用安全
macro_rules! define_event_factory {
    (
        $(
            $(#[$meta:meta])*
            $func_name:ident => (
                $event_type:ident,
                $scope_type:ident,
                |$scope_id:ident, $actor_id:ident $(, $($extra_args:ident),* )?|
                $payload:block
            );
        )*
    ) => {
        pub struct EventFactory;

        impl EventFactory {
            $(
                $(#[$meta])*
                pub fn $func_name(
                    $scope_id: u32,
                    $actor_id: u32,
                    $( $( $extra_args: impl Into<serde_json::Value>, )* )?
                ) -> $crate::event::model::Event {
                    use $crate::event::model::{Event, EventType, EventScope};
                    $crate::event::model::Event {
                        event_id: Event::gen_event_id(),
                        event_type: EventType::$event_type,
                        actor_id: $actor_id,
                        scope_type: EventScope::$scope_type,
                        scope_id: $scope_id,
                        timestamp: chrono::Utc::now().timestamp_millis(),
                        payload: (|| {
                            $( $( let $extra_args = $extra_args.into(); )* )?
                            $payload
                        })(),
                    }
                }
            )*
        }
    };
}
