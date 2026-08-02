===description===
Same false positive as `closure_own_docblock_return_references_class_template.phpt`
but for an arrow function — `analyze_arrow_function` is a separate code path
from `analyze_closure` and had the same template-unaware resolution bug for
its own leading `@return` docblock.
===config===
suppress=UnusedParam
===file===
<?php
namespace PhpOption;

/** @template T */
abstract class Option {
    /** @return T */
    abstract public function get();

    public static function lift(array $args) {
        return array_map(
            /** @return T */
            static fn(self $o) => $o->get(),
            $args
        );
    }
}
===expect===
