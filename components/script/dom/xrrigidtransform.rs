/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::DOMPointBinding::DOMPointInit;
use crate::dom::bindings::codegen::Bindings::DOMPointReadOnlyBinding::DOMPointReadOnlyBinding::DOMPointReadOnlyMethods;
use crate::dom::bindings::codegen::Bindings::XRRigidTransformBinding;
use crate::dom::bindings::codegen::Bindings::XRRigidTransformBinding::XRRigidTransformMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::dompointreadonly::DOMPointReadOnly;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use euclid::{Rotation3D, Transform3D};

#[dom_struct]
pub struct XRRigidTransform {
    reflector_: Reflector,
    position: Dom<DOMPointReadOnly>,
    orientation: Dom<DOMPointReadOnly>,
    #[ignore_malloc_size_of = "defined in euclid"]
    translate: Transform3D<f64>,
    #[ignore_malloc_size_of = "defined in euclid"]
    rotate: Rotation3D<f64>,
}

impl XRRigidTransform {
    fn new_inherited(
        position: &DOMPointReadOnly,
        orientation: &DOMPointReadOnly,
    ) -> XRRigidTransform {
        let translate = Transform3D::create_translation(
            position.X() as f64,
            position.Y() as f64,
            position.Z() as f64,
        );
        let rotate = Rotation3D::unit_quaternion(
            orientation.X() as f64,
            orientation.Y() as f64,
            orientation.Z() as f64,
            orientation.W() as f64,
        );
        XRRigidTransform {
            reflector_: Reflector::new(),
            position: Dom::from_ref(position),
            orientation: Dom::from_ref(orientation),
            translate,
            rotate,
        }
    }

    #[allow(unused)]
    pub fn new(
        global: &Window,
        position: &DOMPointReadOnly,
        orientation: &DOMPointReadOnly,
    ) -> DomRoot<XRRigidTransform> {
        reflect_dom_object(
            Box::new(XRRigidTransform::new_inherited(position, orientation)),
            global,
            XRRigidTransformBinding::Wrap,
        )
    }

    #[allow(unused)]
    pub fn identity(window: &Window) -> DomRoot<XRRigidTransform> {
        let global = window.global();
        let position = DOMPointReadOnly::new(&global, 0., 0., 0., 1.);
        let orientation = DOMPointReadOnly::new(&global, 0., 0., 0., 1.);
        reflect_dom_object(
            Box::new(XRRigidTransform::new_inherited(&position, &orientation)),
            window,
            XRRigidTransformBinding::Wrap,
        )
    }

    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-xrrigidtransform
    pub fn Constructor(
        window: &Window,
        position: &DOMPointInit,
        orientation: &DOMPointInit,
    ) -> Fallible<DomRoot<Self>> {
        let global = window.global();
        let position = DOMPointReadOnly::new_from_init(&global, &position);
        // XXXManishearth normalize this
        let orientation = DOMPointReadOnly::new_from_init(&global, &orientation);
        Ok(XRRigidTransform::new(window, &position, &orientation))
    }
}

impl XRRigidTransformMethods for XRRigidTransform {
    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-position
    fn Position(&self) -> DomRoot<DOMPointReadOnly> {
        DomRoot::from_ref(&self.position)
    }
    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-orientation
    fn Orientation(&self) -> DomRoot<DOMPointReadOnly> {
        DomRoot::from_ref(&self.orientation)
    }
    // https://immersive-web.github.io/webxr/#dom-xrrigidtransform-inverse
    fn Inverse(&self) -> DomRoot<XRRigidTransform> {
        // An XRRigidTransform is a rotation and a translation,
        // i.e. T * R
        //
        // Its inverse is (T * R)^-1
        //    = R^-1 * T^-1
        //    = R^-1 * T^-1 * (R * R^-1)
        //    = (R^-1 * T^-1 * R) * R^-1
        //    = T' * R^-1
        //    = T' * R'
        //
        //  (R^-1 * T^-1 * R) is a translation matrix, and R^-1 is a
        //  rotation matrix, so we can use these in the new rigid transform
        let r_1 = self.rotate.inverse();
        let t_1 = self
            .translate
            .inverse()
            .expect("translation matrices should be invertible");
        let t_p = r_1
            .to_transform()
            .post_mul(&t_1)
            .post_mul(&self.rotate.to_transform());

        let global = self.global();
        let position =
            DOMPointReadOnly::new(&global, t_p.m41.into(), t_p.m42.into(), t_p.m43.into(), 1.);
        let orientation = DOMPointReadOnly::new(
            &global,
            r_1.i.into(),
            r_1.j.into(),
            r_1.k.into(),
            r_1.r.into(),
        );
        XRRigidTransform::new(global.as_window(), &position, &orientation)
    }
}

impl XRRigidTransform {
    pub fn matrix(&self) -> Transform3D<f64> {
        // Spec says the orientation applies first,
        // so post-multiply (?)
        self.translate.post_mul(&self.rotate.to_transform())
    }
}
