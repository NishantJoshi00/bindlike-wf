use std::sync::Arc;

pub trait Workflow<I: Clone>: Clone + Send + Sync {
    type Output: Clone;

    fn current(&self) -> String;

    fn execute(self, input: I) -> Option<Self::Output>;

    fn left(self) -> Option<impl Workflow<I>> {
        None::<()>
    }

    fn right(self) -> Option<impl Workflow<Self::Output>> {
        None::<()>
    }
}

#[derive(Clone)]
pub struct Node<I, O, Ok, Ko> {
    name: String,
    workflow: Arc<dyn Fn(I) -> Option<O> + Send + Sync>,
    ok: Ok,
    ko: Ko,
}

impl<I, O, Ok, Ko> Workflow<I> for Node<I, O, Ok, Ko>
where
    I: Clone,
    O: Clone,
    Ok: Workflow<O>,
    Ko: Workflow<I>,
{
    type Output = O;

    fn current(&self) -> String {
        self.name.clone()
    }

    fn execute(self, input: I) -> Option<Self::Output> {
        (self.workflow)(input)
    }

    fn left(self) -> Option<impl Workflow<I>> {
        Some(self.ko.clone())
    }

    fn right(self) -> Option<impl Workflow<O>> {
        Some(self.ok.clone())
    }
}

impl<I: Clone> Workflow<I> for () {
    type Output = ();

    fn execute(self, _input: I) -> Option<Self::Output> {
        Some(())
    }

    fn current(&self) -> String {
        "End".to_string()
    }
}

impl<I, O, Ko> Node<I, O, (), Ko>
where
    I: Clone,
    Ko: Workflow<I>,
    O: Clone,
{
    pub fn and_then(self, ok: impl Workflow<O>) -> Node<I, O, impl Workflow<O>, Ko> {
        Node {
            name: self.name,
            workflow: self.workflow,
            ok,
            ko: self.ko,
        }
    }
}

impl<I, O, Ok> Node<I, O, Ok, ()>
where
    I: Clone,
    Ok: Workflow<O>,
    O: Clone,
{
    pub fn or_else(self, ko: impl Workflow<I>) -> Node<I, O, Ok, impl Workflow<I>> {
        Node {
            name: self.name,
            workflow: self.workflow,
            ok: self.ok,
            ko,
        }
    }
}

impl<I, O> Node<I, O, (), ()> {
    pub fn new(
        name: &str,
        workflow: impl Fn(I) -> Option<O> + Send + Sync + 'static,
    ) -> Node<I, O, (), ()> {
        Node {
            name: name.to_string(),
            workflow: Arc::new(workflow),
            ok: (),
            ko: (),
        }
    }
}
