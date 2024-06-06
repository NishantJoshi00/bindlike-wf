use crate::workflows::Workflow;

pub trait SyncExecute<I>: Workflow<I>
where
    I: Clone,
{
    fn sync_execute(self, input: I)
    where
        I: Clone;

    fn left(self) -> Option<impl SyncExecute<I>>;

    fn right(self) -> Option<impl SyncExecute<Self::Output>>;
}

impl<W, I> SyncExecute<I> for W
where
    W: Workflow<I>,
    W::Output: Clone,
    I: Clone,
{
    fn sync_execute(self, input: I) {
        let output = self.clone().execute(input.clone());

        if let Some(output) = output {
            <Self as SyncExecute<I>>::right(self).map(|inner| inner.sync_execute(output))
        } else {
            <Self as SyncExecute<I>>::left(self).map(|inner| inner.sync_execute(input))
        }
        .unwrap_or(())
    }

    fn left(self) -> Option<impl SyncExecute<I>> {
        <Self as Workflow<I>>::left(self)
    }

    fn right(self) -> Option<impl SyncExecute<Self::Output>> {
        <Self as Workflow<I>>::right(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::workflows::Node;

    use super::*;

    fn generate_chain(sender: std::sync::mpsc::Sender<Option<usize>>) -> impl SyncExecute<usize> {
        let first_borrow = sender.clone();
        let output = Node::new("check_greater_than_5", move |input: usize| {
            if input > 5 {
                first_borrow.clone().send(Some(input)).unwrap();
                Some(input)
            } else {
                first_borrow.clone().send(None).unwrap();
                None
            }
        });

        let second_borrow = sender.clone();

        let ok = Node::new("multiply_by_2", move |input: usize| {
            second_borrow.clone().send(Some(input * 2)).unwrap();
            Some(input * 2)
        });

        let third_borrow = sender.clone();

        let ko = Node::new("set_to_zero", move |_: usize| {
            third_borrow.clone().send(Some(0)).unwrap();
            Some(0)
        });

        output.and_then(ok).or_else(ko)
    }

    #[test]
    fn test_workflow() {
        let (sender, receiver) = std::sync::mpsc::channel();

        let chain = generate_chain(sender);

        chain.sync_execute(10);

        let results = vec![Some(10), Some(20)];
        let received = receiver.iter().take(2).collect::<Vec<_>>();

        assert_eq!(results, received);
    }
}
