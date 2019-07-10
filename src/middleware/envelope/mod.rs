use crate::context::TransparentContext;

/// Defines the behaviour of an envelope.
/// An envelope allows to get an iterator.
/// The returned iterator allows to iterator over the frames, starting from
/// the current position, and for each frame, returns the envelope value at that frame.
pub trait Envelope<'a, T>: Clone {
    /// The type of the iterator.
    type Iter: Iterator<Item = T>;
    /// Get the iterator.
    fn iter(&'a self) -> Self::Iter;
}


pub trait EnvelopeContext {
    type Marker;
    type Data;
    fn data(&mut self) -> &mut Self::Data;
}

pub trait WithEnvelope<'a, T> 
where T: EnvelopeContext + 'a
{
    fn envelope(&'a mut self, marker: T::Marker) -> &'a mut T::Data;
}


impl<'a, T, U> WithEnvelope<'a, T> for U
where 
    U: TransparentContext<T>,
    T: EnvelopeContext + 'a
{
    fn envelope(&'a mut self, marker: T::Marker) -> &'a mut T::Data {
        self.get().data()
    }
}

mod aftertouch;
