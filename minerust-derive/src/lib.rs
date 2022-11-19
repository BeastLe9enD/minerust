#[macro_export]
macro_rules! protocol {
    ($name: ident, $literal: expr, $id: expr, $(($packet_name: ident, $packet_id: expr, $direction: ident, $state: ident) => $($value_name: ident: $value_type: ty),*),*) => {
        paste::paste! {
            pub struct $name {}

            impl minerust_network::ProtocolVersion for $name {
                fn id() -> i32 {
                    $id
                }

                fn literal() -> &'static str {
                    $literal
                }
            }

            impl $name {
                pub fn packet_ids() -> Vec<u8> {
                    let mut vec = Vec::new();
                    $(
                        vec.push($packet_id);
                    )*
                    vec
                }
            }

            $(
            pub struct $packet_name {
                $(
                $value_name: $value_type,
                )*
            }

            impl minerust_network::connection::Writable for $packet_name {
                fn write(&self, mut buffer: Buffer) -> Result<Buffer, Error> {
                    buffer.write_var_i32($packet_id)?;
                    $(
                    buffer = self.$value_name.write(buffer)?;
                    )*
                    Ok(buffer.clone())
                }
            }

            impl $packet_name {
                pub fn new($($value_name: $value_type,)*) -> Self {
                    Self {
                        $(
                        $value_name,
                        )*
                    }
                }

                pub fn direction() -> minerust_network::PacketDirection {
                    minerust_network::PacketDirection::$direction
                }

                pub fn state() -> minerust_network::PacketState {
                    minerust_network::PacketState::$state
                }

                pub fn id() -> i32 {
                    $id
                }
            }
            )*
        }
    }
}
