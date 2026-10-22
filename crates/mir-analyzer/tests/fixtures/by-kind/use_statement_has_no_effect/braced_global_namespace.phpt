===description===
The braced global namespace is still no namespace: a single-segment `use` there has no effect.
===file===
<?php
namespace {
    use Foo;
//      ^^^ UseStatementHasNoEffect: The use statement with non-compound name 'Foo' has no effect
}
