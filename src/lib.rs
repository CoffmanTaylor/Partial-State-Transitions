use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
    marker::PhantomData,
};

pub trait PartialStateTransition<S, Args> {
    type Striped;

    fn strip(input: S) -> Self::Striped;

    fn partial_call(input: Self::Striped, args: Args) -> Self::Striped;

    fn merge(input: S, result: Self::Striped) -> S;
}

pub struct TransitionCache<T, S, Args>
where
    T: PartialStateTransition<S, Args>,
{
    transition: PhantomData<T>,
    cache: HashMap<(T::Striped, Args), T::Striped>,
}

impl<T, S, Args> TransitionCache<T, S, Args>
where
    T: PartialStateTransition<S, Args>,
{
    pub fn new() -> Self {
        TransitionCache {
            transition: PhantomData,
            cache: HashMap::new(),
        }
    }

    pub fn apply_transition(&mut self, start: S, args: Args) -> S
    where
        S: Clone,
        T::Striped: Clone + Eq + Hash,
        Args: Clone + Eq + Hash,
    {
        let striped = T::strip(start.clone());

        // Check if we already have cached this transition.
        let res_unmerged = match self.cache.entry((striped.clone(), args.clone())) {
            Entry::Occupied(e) => e.get().clone(),
            Entry::Vacant(e) => {
                // We do not already have it. Compute the transition and add it to the map.
                let res = T::partial_call(striped, args);
                e.insert(res.clone());
                res
            }
        };

        T::merge(start, res_unmerged)
    }
}

#[cfg(test)]
mod hand_written {

    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[derive(Clone, PartialEq, Eq, Debug, Hash)]
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

    enum IsEven {}

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

    enum SetNumber {}

    impl PartialStateTransition<Example, usize> for SetNumber {
        type Striped = usize;

        fn strip(input: Example) -> Self::Striped {
            input.field_usize
        }

        fn partial_call(_input: Self::Striped, args: usize) -> Self::Striped {
            args
        }

        fn merge(mut input: Example, result: Self::Striped) -> Example {
            input.field_usize = result;

            input
        }
    }

    #[test]
    fn can_apply_transition_on_cache() {
        let mut cache = TransitionCache::<IsEven, _, _>::new();

        let start = Example::new(42, false);

        let res = cache.apply_transition(start, ());

        assert_eq!(Example::new(42, true), res);
    }

    #[test]
    fn cached_transition_is_only_called_once() {
        static COUNT: AtomicUsize = AtomicUsize::new(0);

        enum IsEven {}

        impl PartialStateTransition<Example, ()> for IsEven {
            type Striped = Example;

            fn strip(input: Example) -> Self::Striped {
                input
            }

            fn partial_call(mut input: Self::Striped, _args: ()) -> Self::Striped {
                COUNT.fetch_add(1, Ordering::SeqCst);
                input.field_bool = input.field_usize % 2 == 0;
                input
            }

            fn merge(_input: Example, result: Self::Striped) -> Example {
                result
            }
        }

        let mut cache = TransitionCache::<IsEven, _, _>::new();

        let start = Example::new(42, false);

        cache.apply_transition(start.clone(), ());
        let res = cache.apply_transition(start, ());

        assert_eq!(Example::new(42, true), res);
        assert_eq!(1, COUNT.load(Ordering::SeqCst));
    }

    #[test]
    fn can_use_transition_on_different_starts() {
        let mut cache = TransitionCache::<IsEven, _, _>::new();

        let start1 = Example::new(42, false);
        let start2 = Example::new(43, true);

        let res1 = cache.apply_transition(start1, ());
        let res2 = cache.apply_transition(start2, ());

        assert_eq!(Example::new(42, true), res1);
        assert_eq!(Example::new(43, false), res2);
    }

    #[test]
    fn transition_with_only_some_fields() {
        let mut cache = TransitionCache::<SetNumber, _, _>::new();

        let start = Example::new(42, false);

        let res = cache.apply_transition(start, 38);

        assert_eq!(Example::new(38, false), res);
    }

    #[test]
    fn partial_transition() {
        static COUNT: AtomicUsize = AtomicUsize::new(0);

        enum SetNumber {}

        impl PartialStateTransition<Example, usize> for SetNumber {
            type Striped = usize;

            fn strip(input: Example) -> Self::Striped {
                input.field_usize
            }

            fn partial_call(_input: Self::Striped, args: usize) -> Self::Striped {
                COUNT.fetch_add(1, Ordering::SeqCst);
                args
            }

            fn merge(mut input: Example, result: Self::Striped) -> Example {
                input.field_usize = result;

                input
            }
        }

        let mut cache = TransitionCache::<SetNumber, _, _>::new();

        cache.apply_transition(Example::new(42, true), 1);
        cache.apply_transition(Example::new(42, false), 1);

        assert_eq!(1, COUNT.load(Ordering::SeqCst));
    }
}
