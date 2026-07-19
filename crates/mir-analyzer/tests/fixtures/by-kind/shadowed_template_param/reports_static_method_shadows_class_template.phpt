===description===
static_call.rs computed an equivalent class_bindings/method_bindings pair to
the instance-call path but never checked for template shadowing — the exact
same shadow via a static call went unreported. Property-parity counterpart
of reports_method_shadows_class_template.phpt.
===config===
suppress=UnusedParam
===file===
<?php
/** @template T */
class Box {}

/** @extends Box<string> */
class StringBox extends Box {
    /**
     * @template T
     * @param T $value
     * @return T
     */
    public static function transform($value) {
        return $value;
    }
}

function test(): void {
    StringBox::transform('hello');
}
===expect===
ShadowedTemplateParam@18:4-18:33: Method template parameter 'T' shadows class-level template parameter with the same name
