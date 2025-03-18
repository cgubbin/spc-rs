use crate::SPCFile;

#[derive(Clone, Debug)]
struct SubFlagParameters(u8);

#[derive(Clone, Debug)]
pub(crate) struct Subheader {
    parameters: SubFlagParameters,
    pub(crate) exponent_y: i8,
    index_number: u16,
    starting_z: f32,
    ending_z: f32,
    noise_value: f32,
    number_points: u32,
    number_co_added_scans: u32,
    w_axis_value: f32,
    reserved: String,
}

pub(crate) struct SubHeaderParser<'a, 'de>(pub(crate) &'a mut SPCFile<'de>);

impl<'a, 'de> SubHeaderParser<'a, 'de> {
    pub(crate) fn parse(&mut self) -> miette::Result<Subheader> {
        let parameters = SubFlagParameters(self.0.read_byte());
        let exponent_y = self.0.read_i8();
        let index_number = self.0.read_u16();
        let starting_z = self.0.read_f32();
        let ending_z = self.0.read_f32();
        let noise_value = self.0.read_f32();
        let number_points = self.0.read_u32();
        let number_co_added_scans = self.0.read_u32();
        let w_axis_value = self.0.read_f32();
        let reserved = self.0.read_unescaped_utf8(4).trim().to_owned();
        Ok(Subheader {
            parameters,
            exponent_y,
            index_number,
            starting_z,
            ending_z,
            noise_value,
            number_points,
            number_co_added_scans,
            w_axis_value,
            reserved,
        })
    }
}
