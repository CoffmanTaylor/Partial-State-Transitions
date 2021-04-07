pub trait PartialStateTransition<S, Args> {
    type Striped;

    fn strip(input: S) -> Self::Striped;

    fn partial_call(input: Self::Striped, args: Args) -> Self::Striped;

    fn merge(input: S, result: Self::Striped) -> S;

    fn call(input: S, args: Args) -> S
    where
        S: Clone,
    {
        let striped = Self::strip(input.clone());
        Self::merge(input, Self::partial_call(striped, args))
    }
}

#[cfg(test)]
mod hand_written {
    use super::*;

    #[derive(Clone, PartialEq, Eq, Debug)]
    struct Example {
        field_usize: usize,
        field_bool: bool,
    }

    impl Example {
        fn new(field_usize: usize, field_bool: bool) -> Self {
            Example {
                field_usize,
                field_bool,
            }
        }
    }

    #[derive(Debug, Hash, PartialEq, Eq)]
    struct IsEven;

    impl PartialStateTransition<Example, ()> for IsEven {
        type Striped = Example;

        fn strip(input: Example) -> Self::Striped {
            input
        }

        fn partial_call(mut input: Self::Striped, _args: ()) -> Self::Striped {
            input.field_bool = input.field_usize % 2 == 0;
            input
        }

        fn merge(_input: Example, result: Self::Striped) -> Example {
            result
        }
    }

    #[test]
    fn direct_call_on_is_even() {
        let start = Example::new(42, false);

        assert_eq!(Example::new(42, true), IsEven::call(start, ()));
    }
}

// #[cfg(test)]
// mod tests {

//     use core::panic;
//     use std::collections::HashMap;

//     #[derive(Clone, Debug, PartialEq, Eq)]
//     struct ExampleComposition {
//         field_usize: Handle<usize>,
//         field_bool: Handle<bool>,
//         field_char: Handle<char>,
//     }

//     #[allow(non_camel_case_types)]
//     #[derive(Clone, PartialEq, Eq, Hash)]
//     struct _ExampleComposition_example_is_even {
//         field_usize: Handle<usize>,
//         field_bool: Handle<bool>,
//     }

//     #[allow(non_camel_case_types)]
//     #[derive(Clone, PartialEq, Eq, Hash)]
//     struct _ExampleComposition_example_not {
//         field_bool: Handle<bool>,
//     }

//     #[allow(non_camel_case_types)]
//     #[derive(Clone, Eq, PartialEq, Hash)]
//     enum _ExampleComposition_Transitions {
//         _ExampleComposition_example_is_even(_ExampleComposition_example_is_even),
//         _ExampleComposition_example_not(_ExampleComposition_example_not),
//     }

//     impl _ExampleComposition_Transitions {
//         fn to_is_even(self) -> _ExampleComposition_example_is_even {
//             match self {
//                 _ExampleComposition_Transitions::_ExampleComposition_example_is_even(v) => v,
//                 _ => panic!("Tried to convert to is_even on non is_even"),
//             }
//         }
//         fn to_not(self) -> _ExampleComposition_example_not {
//             match self {
//                 _ExampleComposition_Transitions::_ExampleComposition_example_not(v) => v,
//                 _ => panic!("Tried to convert to not on non not"),
//             }
//         }
//     }

//     fn example_is_even(
//         input: &_ExampleComposition_example_is_even,
//     ) -> _ExampleComposition_example_is_even {
//         let mut output = input.clone();

//         *output.field_bool.get_mut() = *input.field_usize % 2 == 0;

//         output
//     }

//     fn example_not(input: &_ExampleComposition_example_not) -> _ExampleComposition_example_not {
//         let mut output = input.clone();

//         *output.field_bool.get_mut() = !*input.field_bool;

//         output
//     }

//     fn _strip_example_is_even(start: &ExampleComposition) -> _ExampleComposition_example_is_even {
//         _ExampleComposition_example_is_even {
//             field_bool: start.field_bool.clone(),
//             field_usize: start.field_usize.clone(),
//         }
//     }

//     fn _strip_example_not(start: &ExampleComposition) -> _ExampleComposition_example_not {
//         _ExampleComposition_example_not {
//             field_bool: start.field_bool.clone(),
//         }
//     }

//     fn _merge_example_is_even(
//         result: _ExampleComposition_example_is_even,
//         mut start: ExampleComposition,
//     ) -> ExampleComposition {
//         start.field_usize = result.field_usize;
//         start.field_bool = result.field_bool;

//         start
//     }

//     fn _merge_example_not(
//         result: _ExampleComposition_example_not,
//         mut start: ExampleComposition,
//     ) -> ExampleComposition {
//         start.field_bool = result.field_bool;

//         start
//     }

//     fn call_example_is_even(
//         input: ExampleComposition,
//         memo_cache: &mut HashMap<_ExampleComposition_Transitions, _ExampleComposition_Transitions>,
//     ) -> ExampleComposition {
//         let striped = _strip_example_is_even(&input);

//         // Check if we already have a result for that state.
//         if let Some(result) = memo_cache.get(
//             &_ExampleComposition_Transitions::_ExampleComposition_example_is_even(striped.clone()),
//         ) {
//             _merge_example_is_even(result.clone().to_is_even(), input)
//         } else {
//             let result_unmerged = example_is_even(&striped);
//             memo_cache.insert(
//                 _ExampleComposition_Transitions::_ExampleComposition_example_is_even(striped),
//                 _ExampleComposition_Transitions::_ExampleComposition_example_is_even(
//                     result_unmerged.clone(),
//                 ),
//             );
//             _merge_example_is_even(result_unmerged, input)
//         }
//     }

//     fn call_example_not(
//         input: ExampleComposition,
//         memo_cache: &mut HashMap<_ExampleComposition_Transitions, _ExampleComposition_Transitions>,
//     ) -> ExampleComposition {
//         let striped = _strip_example_not(&input);

//         // Check if we already have a result for that state.
//         if let Some(result) = memo_cache
//             .get(&_ExampleComposition_Transitions::_ExampleComposition_example_not(striped.clone()))
//         {
//             _merge_example_not(result.clone().to_not(), input)
//         } else {
//             let result_unmerged = example_not(&striped);
//             memo_cache.insert(
//                 _ExampleComposition_Transitions::_ExampleComposition_example_not(striped),
//                 _ExampleComposition_Transitions::_ExampleComposition_example_not(
//                     result_unmerged.clone(),
//                 ),
//             );
//             _merge_example_not(result_unmerged, input)
//         }
//     }

//     #[test]
//     fn can_strip_and_merge() {
//         let start = ExampleComposition {
//             field_usize: Handle::new_unifying_state(42),
//             field_bool: Handle::new_unifying_state(false),
//             field_char: Handle::new_unifying_state('a'),
//         };

//         let mut memo_cache = HashMap::new();

//         let result = call_example_is_even(start.clone(), &mut memo_cache);

//         assert_eq!(
//             ExampleComposition {
//                 field_usize: start.field_usize.clone(),
//                 field_char: start.field_char.clone(),
//                 field_bool: start.field_bool.add_unifying_state(true),
//             },
//             result
//         );
//     }
// }
