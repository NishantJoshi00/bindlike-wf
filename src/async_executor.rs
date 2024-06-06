use std::thread;

use crate::workflows::Workflow;

trait AsyncExecute<I>: Workflow<I>
where
    I: Clone,
{
    fn async_execute<R: Registry + Clone + WaitTime + 'static>(self, registry: R)
    where
        I: Clone;

    fn left(self) -> Option<impl AsyncExecute<I>>;

    fn right(self) -> Option<impl AsyncExecute<Self::Output>>;
}

trait Registry: Send + Sync {
    fn fetch<I: Clone>(&self, name: String) -> impl Iterator<Item = I>;
    fn store<I: Clone>(&self, name: String, item: Box<I>);
}

trait WaitTime {
    fn get_wait_time(&self) -> core::time::Duration;
}

impl<W, I> AsyncExecute<I> for W
where
    W: Workflow<I> + 'static,
    W::Output: Clone,
    I: Clone + 'static,
{
    fn async_execute<R: Registry + Clone + WaitTime + 'static>(self, registry: R) {
        let left = <Self as AsyncExecute<I>>::left(self.clone())
            .map(|inner| inner.async_execute(registry.clone()));

        let right = <Self as AsyncExecute<I>>::right(self.clone())
            .map(|inner| inner.async_execute(registry.clone()));

        left.zip(right).map(|_| ()).unwrap_or(());

        thread::spawn(move || loop {
            let wait_time = registry.get_wait_time();
            thread::sleep(wait_time);

            registry.fetch(self.current()).for_each(|input: I| {
                let output = self.clone().execute(input.clone());

                match (output, self.clone().left(), self.clone().right()) {
                    (Some(output), _, Some(right)) => {
                        registry.store(right.current(), Box::new(output))
                    }
                    (None, Some(left), _) => registry.store(left.current(), Box::new(input)),
                    _ => (),
                }
            })
        });
    }

    fn left(self) -> Option<impl AsyncExecute<I>> {
        <Self as Workflow<I>>::left(self)
    }

    fn right(self) -> Option<impl AsyncExecute<Self::Output>> {
        <Self as Workflow<I>>::right(self)
    }
}
