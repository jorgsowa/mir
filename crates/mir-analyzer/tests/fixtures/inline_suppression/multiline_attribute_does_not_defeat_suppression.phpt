===description===
A multi-line PHP 8 attribute (`#[` opener on its own line, closing `]`
several lines later) previously had no bracket-depth tracking at all —
the opener line matched neither the single-line attribute-skip predicate
nor the comment-skip predicate, so it became the wrongly-chosen
suppression target instead of the class declaration after it.
===config===
suppress=UndefinedAttributeClass
===file===
<?php
/** @mir-ignore UndefinedClass */
#[
    Bar(name: 'x')
]
class Foo extends UndefinedClass {
}
===expect===
