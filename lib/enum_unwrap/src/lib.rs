//
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
// ВНИМАНИЕ: ГОВНОКОД СГЕНЕРИРОВАН НЕЙРОСЕТЬЮ
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
//
//
// мне было лень
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Макрос `UnwrapEnum` автоматически генерирует методы для распаковки
/// вариантов enum или возвращения None.
///
/// Для каждого варианта enum с единственным содержимым генерируется метод
/// с именем в нижнем регистре, который возвращает Option<T>, где T - тип значения варианта.
///
/// # Пример
///
/// ```
/// use unwrap_enum_macro::UnwrapEnum;
///
/// #[derive(UnwrapEnum)]
/// pub enum Property {
///     Int(i64),
///     String(String),
///     Float(f64),
/// }
///
/// fn main() {
///     let p = Property::Int(42);
///     assert_eq!(p.to_int(), Some(42));
///     assert_eq!(p.to_string(), None);
///     
///     let p = Property::String("hello".to_string());
///     assert_eq!(p.to_string().as_deref(), Some("hello"));
/// }
/// ```
#[allow(clippy::needless_doctest_main)]
#[proc_macro_derive(UnwrapEnum)]
pub fn derive_unwrap_enum(input: TokenStream) -> TokenStream {
    // Парсим входной поток токенов в AST
    let input = parse_macro_input!(input as DeriveInput);

    // Получаем имя типа
    let name = &input.ident;

    // Убеждаемся, что макрос применен к enum
    let data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("UnwrapEnum может быть применен только к enum"),
    };

    // Создаем функции для каждого варианта enum
    let mut unwrap_functions = Vec::new();

    for variant in data.variants.iter() {
        let variant_name = &variant.ident;
        let method_name = format_ident!("to_{}", variant_name.to_string().to_lowercase());

        // Обрабатываем только варианты с одним содержимым
        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field_type = &fields.unnamed.first().unwrap().ty;

                // Создаем функцию распаковки для этого варианта
                let unwrap_fn = quote! {
                    pub fn #method_name(&self) -> Option<&#field_type> {
                        match self {
                            #name::#variant_name(value) => Some(value),
                            _ => None,
                        }
                    }
                };

                unwrap_functions.push(unwrap_fn);
            }
            _ => {
                // Пропускаем варианты без содержимого или с несколькими полями
                continue;
            }
        }
    }

    // Генерируем реализацию
    let expanded = quote! {
        impl #name {
            #(#unwrap_functions)*
        }
    };

    // Возвращаем сгенерированный код
    expanded.into()
}
