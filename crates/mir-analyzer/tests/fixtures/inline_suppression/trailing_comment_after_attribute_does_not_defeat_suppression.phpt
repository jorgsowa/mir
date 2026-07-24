===description===
A sibling gap to the attribute-between-suppression-and-target fix: a
trailing same-line comment after a single-line `#[Attr]` (`#[Bar] //
note`) made `is_attribute_only` require `]` to be the line's last
character, so the line matched neither the attribute-skip nor the
comment-skip predicate and became the wrongly-chosen suppression target
instead of the class declaration after it.
===config===
suppress=UndefinedAttributeClass
===file===
<?php
/** @mir-ignore UndefinedClass */
#[Bar] // some note
class Foo extends UndefinedClass {
}
===expect===
