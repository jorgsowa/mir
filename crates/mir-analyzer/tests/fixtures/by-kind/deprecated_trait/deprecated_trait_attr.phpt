===description===
Sibling of deprecated_trait: trait.rs only read the docblock tag, missing
the #[Deprecated] attribute fallback class.rs has.
===file===
<?php
#[\Deprecated]
trait T {}

class C {
    use T;
}
===expect===
DeprecatedTrait@5:0-5:9: Trait T is deprecated
