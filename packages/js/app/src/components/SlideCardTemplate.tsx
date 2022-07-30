import React from 'react';
import { View, ViewStyle, PanResponderInstance } from 'react-native';
import Styles from './Styles';

const SlideCardTemplate: React.FunctionComponent<{
    children: JSX.Element;
    style?: ViewStyle;
    panResponder: PanResponderInstance;
}> = ({ children, style, panResponder }) => (
    <View
        style={{
            ...Styles.slideCardTemplate,
            ...style,
        }}
        {...panResponder.panHandlers}
    >
        <View
            style={{
                width: '100%',
                alignItems: 'center',
                justifyContent: 'center',
            }}
        >
            <View
                style={{
                    width: 64,
                    height: 8,
                    backgroundColor: '#d3d3d3',
                    borderRadius: 100,
                }}
            ></View>
        </View>
        {children}
    </View>
);

export default SlideCardTemplate;
