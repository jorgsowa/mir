===description===
FN: property declarations never consulted #[LanguageLevelTypeAware], so
Exception::$file/$line lost their PHP-8.1+ refined string/int type.
===config===
suppress=UnusedVariable
===file===
<?php
class MyException extends Exception {
    public function test(): void {
        $f = $this->file;
        /** @mir-check $f is string */
        $l = $this->line;
        /** @mir-check $l is int */
        echo '';
    }
}
===expect===
