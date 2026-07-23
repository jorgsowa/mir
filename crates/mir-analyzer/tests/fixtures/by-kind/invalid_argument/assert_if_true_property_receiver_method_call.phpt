===description===
@psalm-assert-if-true on a method call previously only narrowed when the
receiver was a bare variable (method_call_receiver_fqcn only tried
extract_var_name) — a property-access receiver (`$this->validator->isInt`,
a very common real-world shape) silently no-oped the whole assertion.
===config===
suppress=MissingConstructor
===file===
<?php
class Validator {
    /**
     * @param mixed $p
     * @psalm-assert-if-true int $p
     */
    public function isInt($p): bool {
        return is_int($p);
    }
}
class Holder {
    public Validator $validator;
}
/**
 * @param mixed $p
 */
function doWork(Holder $h, $p): void {
    if ($h->validator->isInt($p)) {
        strlen($p);
    }
}
===expect===
ArgumentTypeCoercion@19:15-19:17: Argument $string of strlen() expects 'string', got 'int' — coercion may fail at runtime
