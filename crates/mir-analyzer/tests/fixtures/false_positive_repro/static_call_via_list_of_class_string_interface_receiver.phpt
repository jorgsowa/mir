===description===
The exact reported shape: a list<class-string<Interface>> parameter's
foreach-bound element used as a :: receiver — nesting the class-string one
level deeper than a bare parameter must not change whether the interface
check fires.
===config===
suppress=MixedAssignment
===file===
<?php

interface Provider {
    public static function getDefinitions(): array;
}
class Registry {
    /** @param list<class-string<Provider>> $providers */
    public function __construct(array $providers) {
        foreach ($providers as $provider) {
            foreach ($provider::getDefinitions() as $d) {
                echo $d;
            }
        }
    }
}
===expect===
