===description===
invalidNativeIntersectionArgument
===file===
<?php
                    interface A {
                        function foo(): void;
                    }
                    interface B {
                    }
                    class C implements A {
                        function foo(): void {
                        }
                    }
                    function test(A&B $in): void {
                        $in->foo();
                    }
                    test(new C());
                
===expect===
InvalidArgument
===ignore===
TODO
