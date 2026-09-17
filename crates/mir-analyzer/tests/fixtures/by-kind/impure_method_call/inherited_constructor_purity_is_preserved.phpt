===description===
An inherited constructor keeps its own purity when called through an
intermediate parent class that does not redeclare it.
===config===
suppress=UnusedParam
===file===
<?php
class BaseException {
    /** @pure */
    public function __construct(string $message) {}
}

class IntermediateException extends BaseException {}

class DomainException extends IntermediateException {
    /** @pure */
    public function __construct(string $message) {
        parent::__construct($message);
    }
}
===expect===
