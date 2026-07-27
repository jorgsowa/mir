===description===
method_call_receiver_fqcn only resolved a bare-variable or 1-hop
property-access receiver — a 2-hop chain (`$this->service->validator`)
silently no-oped the whole @psalm-assert-if-true dispatch, unlike the
1-hop case (`$this->validator->isInt($p)`) which already worked.
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
class Service {
    public Validator $validator;
}
class Holder {
    public Service $service;
}
/**
 * @param mixed $p
 */
function doWork(Holder $h, $p): void {
    if ($h->service->validator->isInt($p)) {
        strlen($p);
    }
}
===expect===
ArgumentTypeCoercion@22:15-22:17: Argument $string of strlen() expects 'string', got 'int' — coercion may fail at runtime
