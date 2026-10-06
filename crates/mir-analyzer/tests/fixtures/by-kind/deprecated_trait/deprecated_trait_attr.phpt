===description===
Sibling of deprecated_trait: trait.rs only read the docblock tag, missing
the #[Deprecated] attribute fallback class.rs has.
===file===
<?php
#[\Deprecated]
trait T {}

class C {
//<^^^^^^^^^ DeprecatedTrait: Trait T is deprecated
    use T;
}
