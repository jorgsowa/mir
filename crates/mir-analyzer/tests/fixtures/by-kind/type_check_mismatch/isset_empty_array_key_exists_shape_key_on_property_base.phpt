===description===
isset()/!empty()/array_key_exists() shape-key narrowing on a nested array
access now also works when the array base is a property receiver
(`$this->data['key']`), not just a plain variable.
===config===
suppress=UnusedParam,UnusedVariable,MissingConstructor,MixedArgument
===file===
<?php
class Config {
    /** @var array{name?: string} */
    public array $data = [];

    /** @var array{sub?: array{name?: string}} */
    public array $nested = [];

    public function issetNarrowsProp(): string {
        if (isset($this->data['name'])) {
            /** @mir-check $this->data is array{name: string} */
            $_ = 1;
            return $this->data['name'];
        }
        return 'default';
    }

    public function notEmptyNarrowsProp(): string {
        if (!empty($this->data['name'])) {
            /** @mir-check $this->data is array{name: non-empty-string} */
            $_ = 1;
            return $this->data['name'];
        }
        return 'default';
    }

    public function arrayKeyExistsNarrowsProp(): string {
        if (array_key_exists('name', $this->data)) {
            /** @mir-check $this->data is array{name: string} */
            $_ = 1;
            return $this->data['name'];
        }
        return 'default';
    }

    public function arrayKeyExistsNarrowsNestedPropBase(): string {
        if (array_key_exists('name', $this->nested['sub'])) {
            /** @mir-check $this->nested is array{sub: array{name: string}} */
            $_ = 1;
            return $this->nested['sub']['name'];
        }
        return 'default';
    }
}
===expect===
PossiblyNullArgument@37:37-37:57: Argument $array of array_key_exists() might be null
