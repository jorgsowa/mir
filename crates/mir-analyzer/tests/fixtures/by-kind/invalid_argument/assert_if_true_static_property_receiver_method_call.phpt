===description===
@psalm-assert-if-true on a method call reached through a static-property
receiver (`self::$validator->isInt($p)`) — method_call_receiver_fqcn only
ever tried a bare variable or an instance-property chain, silently
no-oping the whole assertion for a static-property receiver, even though
that shape is already a first-class case for the assertion's TARGET side
elsewhere in this file.
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
    private static Validator $validator;

    /**
     * @param mixed $p
     */
    public function doWork($p): void {
        if (self::$validator->isInt($p)) {
            strlen($p);
        }
    }
}
===expect===
ArgumentTypeCoercion@19:19-19:21: Argument $string of strlen() expects 'string', got 'int' — coercion may fail at runtime
