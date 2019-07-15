use crate::context::TransparentContext;

pub trait EnvelopeContext {
    type Marker;
    type Data;
    fn data(&mut self) -> &mut Self::Data;
}

pub trait WithEnvelope<'a, T>
where
    T: EnvelopeContext + 'a,
{
    fn envelope(&'a mut self, marker: T::Marker) -> &'a mut T::Data;
}

impl<'a, T, U> WithEnvelope<'a, T> for U
where
    U: TransparentContext<T>,
    T: EnvelopeContext + 'a,
{
    fn envelope(&'a mut self, _marker: T::Marker) -> &'a mut T::Data {
        self.get().data()
    }
}

pub mod aftertouch;
