===description===
`{@inheritdoc}` param types follow the parameter name when the child inserts a parameter.
===file===
<?php
class P {
    /**
     * @param int $code
     * @param array $headers
     */
    public function __construct($code = 0, $headers = []) {
        echo $code, count($headers);
    }
}
class C extends P {
    /** {@inheritdoc} */
    public function __construct(string $extra, $code = 0) {
        /** @mir-check $code is int */
        parent::__construct($code);
        echo $extra;
    }
}
