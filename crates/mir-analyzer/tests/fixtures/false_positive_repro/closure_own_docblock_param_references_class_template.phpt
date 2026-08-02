===description===
Same bug as `closure_own_docblock_return_references_class_template.phpt` but
for a closure literal's OWN leading `@param` docblock tag — `apply_doc_param_types`
resolved it through the template-unaware `resolve_named_objects_in_union`, so a
bare `T` matching the enclosing class's `@template` became an ordinary (and
namespace-mis-qualified) class reference instead of a `TTemplateParam`.
===config===
suppress=UnusedParam,UnusedVariable,MissingClosureReturnType
===file===
<?php

/** @template T */
abstract class Option {
    public static function each(array $args) {
        array_map(
            /** @param T $item */
            static function ($item) {
                /** @mir-check $item is T */
                $_ = $item;
            },
            $args
        );
    }
}
===expect===
