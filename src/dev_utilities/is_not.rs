/// A trait used to avoid overlapping implementations.
/// 
/// # Examples
/// The following code does not compile (TODO: give a reference to the compiler
/// documentation):
///
/// ```ignore
/// # // Gives error "conflicting implementations".
/// trait SomeTrait<T> { /* ... */ }
/// struct SpecialType { /* ... */ }
/// 
/// struct MyStruct { /* ... */ }
/// 
/// impl SomeTrait<SpecialType> for MyStruct {
///     // special treatment
/// }
///
/// impl<T> SomeTrait<T> for MyStruct {
///     // default treatment
/// }
/// ```
///
/// Using the `IsNot` trait, we can circumvent this restriction of the language:
///
/// ```
/// use rsynth::dev_utilities::is_not::IsNot;
/// trait SomeTrait<T> { /* ... */ }
/// struct SpecialType1 { /* ... */ }
/// impl IsNot<SpecialType2> for SpecialType1 {}
/// struct SpecialType2 { /* ... */ }
/// impl IsNot<SpecialType1> for SpecialType2 {}
///
/// struct MyStruct { /* ... */ }
///
/// impl SomeTrait<SpecialType1> for MyStruct {
///     // special treatment
/// }
///
/// impl<T> SomeTrait<T> for MyStruct 
/// where SpecialType1: IsNot<T>
/// {
///     // default treatment
/// }
/// ```
///
/// In this example, because `SpecialType` does not implement 
/// `IsNot<SpecialType>` and hence the compiler knows that the impls do not
/// overlap.
///
/// # Usage and restrictions for libraries
///
/// You will typically only use `IsNot` when you are defining your own "special
/// types". When you do so, the following instructions and pitfalls apply.
///
/// ## Implementing `IsNot` for each pair of different types.
/// Within a library crate, each type that you want to use for the given purpose,
/// needs to implement `IsNot<T>` for every other data type `T` that you want
/// to use for that purpose.
///
/// When the library crate can be extended by defining other data types used
/// for this purpose in other crates, each crate typically defines a
/// "marker trait":
/// ```
/// use rsynth::dev_utilities::is_not::IsNot;
/// trait IsNotInLibraryCrate {}
/// ```
/// This "marker traits" are then used as follows:
///
/// ```
/// use rsynth::dev_utilities::is_not::IsNot;
/// # trait IsNotInLibraryCrate {}
/// struct SpecialType1 { /* ... */ }
/// impl IsNot<SpecialType2> for SpecialType1 {}
/// impl<U> IsNot<U> for SpecialType1 where U: IsNotInLibraryCrate {}
///
/// struct SpecialType2 { /* ... */ }
/// impl IsNot<SpecialType1> for SpecialType2 {}
/// impl<U> IsNot<U> for SpecialType2 where U: IsNotInLibraryCrate {}
/// ```
/// 
/// ## Restriction on generics (1)
/// 
/// TODO: write this section
/// ```ignore
/// # // Does not compile because of conflicting implementations.
/// use rsynth::dev_utilities::is_not::IsNot;
/// trait SomeTrait<T> { /* ... */ }
/// struct MyStruct<T> { /* ... */ }
/// impl<U> SomeTrait<U> for MyStruct<U> {
///   // ...
/// }
/// impl<T, U> SomeTrait<T> for MyStruct<U> where T: IsNot<U> {
///   // ...
/// }
/// ```
/// TODO: explain how this can be solved by "downcasting".
///
/// ## Restriction on generics (2)
/// Unfortunately, it is not typically not possible to do something similar 
/// to the following:
/// ```ignore
/// # // Ignoring because it has conflicting implementations.
/// use rsynth::dev_utilities::is_not::IsNot;
/// struct MyStruct<U> { 
/// # dummy: U
///     /* ... */ 
/// }
///
/// trait SomeTrait<T> { /* ... */ }
/// struct SpecialType<U> { /* ... */ }
/// impl<T, U> IsNot<SpecialType<U>> for SpecialType<T> {}
///
/// impl<U> SomeTrait<SpecialType<U>> for MyStruct<U> {
///     // special treatment
/// }
///
/// impl<T, U> SomeTrait<T> for MyStruct<U>
/// where T: IsNot<SpecialType<U>>
/// {
///     // default treatment
/// }
/// ```
/// The reason for this is that another crate may define a datatype
/// `OtherDataType` and implement `IsNot<OtherDataType>` for this.
/// When this happens, `SpecialType<OtherDatatype>` implements
/// `IsNot<SpecialType<OtherDatatype>>` and both impls apply for
/// `SpecialType<OtherDatatype>`
///
/// 
/// ### The specific trait solution
/// In some situations, you can solve this problem by defining a
/// special trait related to the data type you want a special implementation
/// for. This trait can define a function that returns an `Option`, the result
/// of which can be used at run time to check if the given type is the special
/// type. In its implementation of the special trait, the special type returns
/// `Some(...)` and (blanket) impls are provided that returns `None` for 
/// "all other" types.
/// 
/// ```
/// use rsynth::dev_utilities::is_not::IsNot;
/// trait SomeTrait<T> { 
///     fn trait_function(&self, parameter: T);
/// }
///
/// struct SomeType1 {}
/// impl<U> IsNot<SpecialType<U>> for SomeType1 {}
///
/// struct SpecialType<U> { 
/// #   dummy: U
///     /* ... */ 
/// }
/// # struct SomeInfoType {}
/// trait SpecialTrait {
///     // Return `None` when the type is not SpecialType<U>
///     fn get_special_info(&self) -> Option<SomeInfoType> {
///         None // default implementation
///     }
/// }
/// 
/// impl<T> SpecialTrait for SpecialType<T> {
///     fn get_special_info(&self) -> Option<SomeInfoType> {
///         Some(
///             // ...
/// #           unimplemented!()
///         )
///     }
/// }
/// impl SpecialTrait for SomeType1 {} // default impl
/// // Also blanket impl for types not defined in this crate, using
/// // the "marker traits" as explain under
/// // "##Implementing `IsNot` for each pair of different types."
///
/// struct MyStruct { /* ... */ }
/// impl<T: SpecialTrait> SomeTrait<T> for MyStruct {
///     fn trait_function(&self, parameter: T) {
///         if let Some(info) = parameter.get_special_info() {
///             // special treatment
///         } else {
///             // default treatment
///         }
///     }
/// }
/// ```

pub trait IsNot<T> {}

/// A trait that is not implemented for anything in the rsynth crate.
/// You should implement this for all events that you define in another crate.
pub trait NotInRSynth {}
