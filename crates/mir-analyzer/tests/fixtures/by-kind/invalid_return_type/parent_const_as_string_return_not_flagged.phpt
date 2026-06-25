===description===
parent::STRING_CONST returned from a function declared : string is not flagged.
parent:: now resolves to the actual constant type instead of mixed.
===file===
<?php
class Config {
    const ENV = 'production';
}
class AppConfig extends Config {
    public function environment(): string {
        return parent::ENV;
    }
}
===expect===
