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

pub fn apply_transition<T, S, Args>(start: S, args: Args) -> S
where
    T: PartialStateTransition<S, Args>,
    S: Clone,
{
    let stripped = T::strip(start.clone());
    T::merge(start, T::partial_call(stripped, args))
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

    #[test]
    fn direct_call_on_is_even() {
        let start = Example::new(42, false);

        assert_eq!(
            Example::new(42, true),
            apply_transition::<IsEven, _, _>(start, ())
        );
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
}
