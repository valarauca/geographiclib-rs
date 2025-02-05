

sed -e 's/\([^\s]\+\) \([^\s]\+\) \([^\s]\+\) \([^\s]\+\) \([^\s]\+\) \([^\s]\+\) \([^\s]\+\) \([^\s]\+\) \([^\s]\+\) \([^\s]\+\)/TestData { lat1: \1, lon1: \2, azimuth: \3, lat2: \4, lon2: \5, reverse_azimuth: \6, distance: \7, arc_distance: \8, reduced_length: \9, area_under: \10 },/g'
