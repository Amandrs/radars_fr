#!/usr/bin/perl -w

# https://gordthompson.github.io/ov2optimizer/ov2FileFormat.html
# https://www.tomtomforums.com/threads/ov2-file-format.14454/
# https://perldoc.perl.org/functions/pack
# https://www.poieditor.com/

use autodie;

my $stop = false;
open IN, '<:raw' , 'test.ov2' || die $!;
binmode(IN);
while(1) {
	my $buf = '';
	my $ok = read IN, $buf, 1;
	#print "read $ok byte(s)\n";
	last if not defined $ok or $ok == 0;
	my $type = unpack 'C', $buf;
	#print "Type \"$type\" \n"; 
	if ($type == 1) {
		print("Skip\n");
		seek(IN,20,1); #+20
		$stop=true;
	} elsif($type ==2) {
		read IN, $buf, 12;
		my ($len,$lng,$lat) = unpack 'V V V', $buf;
		read IN, $buf, ($len-13);
		my $msg = unpack 'a*', $buf;
		print("Poi (",$lng/100000,",",$lat/100000,") \"$msg\"\n");
	} else {
		print("Unhandled type ... exiting !");
		last;
	}
}