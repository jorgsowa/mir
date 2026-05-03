===description===
theresNoCatch
===file===
<?php
                    function missing_return() : bool {
                        try {
                        } finally {
                        }
                    }
===expect===
InvalidReturnType
===ignore===
TODO
