// Tweaking variables is a common occourance. Anything that would be a non-obvious constant
// (ie loop 10 times doesn't need to go here) should be contained here for easy updating.

// Magic numbers are the devil.
// Values here may change during runtime, and are stored/retrieved from the database if they exist.
pub(crate) mod terms_and_conditions;
pub(crate) mod roles;
pub(crate) mod guild;
pub(crate) mod channels;
pub(crate) mod formatting;