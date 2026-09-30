===description===
Only implements one requirement
===file===
<?php
use ImplementationRequirementsTraitImposesImplementationRequirements;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ParseError: Parse error: The use statement with non-compound name 'ImplementationRequirementsTraitImposesImplementationRequirements' has no effect
use ImplementationRequirementsBaseA;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ParseError: Parse error: The use statement with non-compound name 'ImplementationRequirementsBaseA' has no effect

class Invalid implements A {
//                       ^ UndefinedClass: Class A does not exist
    use ImposesImplementationRequirements;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedTrait: Trait ImposesImplementationRequirements does not exist
}

===expect===
