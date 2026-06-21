===description===
FP-C: encoding builtins (mb_convert_encoding, iconv) return string|false in stubs,
but false only occurs on programming errors (invalid encoding). Normal usage should
not emit InvalidPropertyAssignment or NullableReturnStatement.
===config===
suppress=UnusedVariable,UnusedParam
php_version=8.2
===file===
<?php

class Converter {
    public string $result = '';

    public function convert(string $s): string {
        $this->result = mb_convert_encoding($s, 'UTF-8', 'ISO-8859-1');
        return $this->result;
    }

    public function iconv_convert(string $s): string {
        return iconv('ISO-8859-1', 'UTF-8', $s);
    }
}

function mb_to_utf8(string $s): string {
    return mb_convert_encoding($s, 'UTF-8');
}

function check_result(string $s): void {
    $result = mb_convert_encoding($s, 'UTF-8');
    /** @mir-check $result is string */
    echo $result;
}
===expect===
