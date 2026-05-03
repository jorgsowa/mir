===description===
missingTemplateExtendsNativeMultipleInterface
===file===
<?php
                    /**
                     * @extends Iterator<mixed, mixed>
                     */
                    interface a extends Iterator, Traversable {
                    }
                
===expect===
MissingTemplateParam
===ignore===
TODO
