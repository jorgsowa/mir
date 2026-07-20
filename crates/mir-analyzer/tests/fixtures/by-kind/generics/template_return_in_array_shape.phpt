===description===
G2: a method-level template param nested inside an `array{...}` shape
`@param` type is recognized during per-call template-binding inference —
`substitute_template_params` had no `TKeyedArray` arm, so a template name
inside a shape leaked through as a bogus, unresolved `TNamedObject` atom
instead of becoming a real `TTemplateParam`, which `infer_template_bindings`
requires to recognize an argument position as inferrable at all.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
class Box {
    /**
     * @template TKey
     * @param array{0: TKey} $pair
     * @return TKey
     */
    public function first($pair) {
        return $pair[0];
    }
}

$b = new Box();
$k = $b->first([1]);
/** @mir-check $k is 1 */
$k;
===expect===
