use crate::{
    problem::direct::{
        arg_value::{DirectProblemDistanceArg,DistanceIn,ArcIn,AzimuthOnly},
    },
};

pub(in crate) mod geod_test_100;

#[derive(Clone,Debug)]
pub(in crate) struct TestData {
    pub(in crate) lat1: f64,
    pub(in crate) lon1: f64,
    pub(in crate) azimuth: f64,
    pub(in crate) lat2: f64,
    pub(in crate) lon2: f64,
    pub(in crate) reverse_azimuth: f64,
    pub(in crate) distance: f64,
    pub(in crate) arc_distance: f64,
    pub(in crate) reduced_length: f64,
    pub(in crate) area_under: f64,
    pub(in crate) ident: &'static str,
}
impl TestData {
    pub(in crate) fn direct_tests(&self) -> (DirectTest<DistanceIn>,DirectTest<DistanceIn>,DirectTest<ArcIn>,DirectTest<ArcIn>) {
        let v1 = DirectTest {
            lat1: self.lat1,
            lon1: self.lon1,
            azi: AzimuthOnly::from(self.azimuth),
            dist: DistanceIn::from(self.distance),

            lat2: self.lat2,
            lon2: self.lon2,
            s12: self.distance,
            azi2: self.reverse_azimuth,
            a12: self.arc_distance,
            m12: self.reduced_length,
            area: self.area_under,
            ident: self.ident,
        };
        let v2 = DirectTest {
            lat1: self.lat2,
            lon1: self.lon2,
            azi: AzimuthOnly::from(self.reverse_azimuth),
            dist: DistanceIn::from(self.distance),

            lat2: self.lat1,
            lon2: self.lon1,
            azi2: self.azimuth, 
            s12: self.distance,
            a12: self.arc_distance,
            m12: self.reduced_length,
            area: self.area_under,
            ident: self.ident,
        };
        let v3 = DirectTest {
            lat1: self.lat1,
            lon1: self.lon1,
            azi: AzimuthOnly::from(self.azimuth),
            dist: ArcIn::from(self.arc_distance),

            lat2: self.lat2,
            lon2: self.lon2,
            azi2: self.reverse_azimuth,
            s12: self.distance,
            a12: self.arc_distance,
            m12: self.reduced_length,
            area: self.area_under,
            ident: self.ident,
        };
        let v4 = DirectTest {
            lat1: self.lat2,
            lon1: self.lon2,
            azi: AzimuthOnly::from(self.reverse_azimuth),
            dist: ArcIn::from(self.arc_distance),

            lat2: self.lat1,
            lon2: self.lon1,
            azi2: self.azimuth, 
            s12: self.distance,
            a12: self.arc_distance,
            m12: self.reduced_length,
            area: self.area_under,
            ident: self.ident,
        };
        (v1,v2,v3,v4)
    }
}


pub(in crate) struct DirectTest<D>
where
    D: DirectProblemDistanceArg
{
    // arguments in
    pub(in crate) lat1: f64,
    pub(in crate) lon1: f64,
    pub(in crate) azi: AzimuthOnly,
    pub(in crate) dist: D,

    // arguments out (that we test against)
    pub(in crate) lat2: f64,
    pub(in crate) lon2: f64,
    pub(in crate) azi2: f64,
    pub(in crate) s12: f64,
    pub(in crate) a12: f64,
    pub(in crate) m12: f64,
    pub(in crate) ident: &'static str,

    // libraries seem to generate really bad values for this?
    #[allow(dead_code)]
    pub(in crate) area: f64,
}
